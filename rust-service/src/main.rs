use std::env;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DEFAULT_PORT: u16 = 9000;

fn main() -> std::io::Result<()> {
    let service_name = env_or("SERVICE_NAME", "rust-service");
    let shared_dir = PathBuf::from(env_or("SHARED_DIR", "/shared"));
    let port = env::var("RUST_SERVICE_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT);

    bootstrap(&shared_dir, &service_name)?;

    let listener = TcpListener::bind(("0.0.0.0", port))?;
    println!("{} listening on 0.0.0.0:{}", service_name, port);

    loop {
        match listener.accept() {
            Ok((stream, _addr)) => {
                if let Err(err) = handle_client(stream, &service_name, &shared_dir) {
                    eprintln!("request error: {err}");
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(err) => {
                eprintln!("connection error: {err}");
                std::thread::sleep(Duration::from_millis(200));
            }
        }
    }
}

fn env_or(name: &str, fallback: &str) -> String {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

fn bootstrap(shared_dir: &Path, service_name: &str) -> std::io::Result<()> {
    fs::create_dir_all(shared_dir)?;
    append_record(shared_dir, service_name, "boot")
}

fn append_record(shared_dir: &Path, service_name: &str, marker: &str) -> std::io::Result<()> {
    let path = shared_dir.join(format!("{service_name}.txt"));
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;

    writeln!(file, "{} | {} | {}", unix_timestamp(), service_name, marker)
}

fn handle_client(
    mut stream: TcpStream,
    service_name: &str,
    shared_dir: &Path,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;

    let mut request_parts = first_line.split_whitespace();
    let method = request_parts.next().unwrap_or("");
    let path = request_parts.next().unwrap_or("/");

    let (status, body) = match (method, path) {
        ("GET", "/health") => ("200 OK", json_response(service_name, "ok", None)),
        ("POST", "/write") => {
            append_record(shared_dir, service_name, "manual write")?;
            ("201 Created", json_response(service_name, "written", None))
        }
        ("GET", "/shared") => ("200 OK", shared_snapshot(service_name, shared_dir)?),
        _ => (
            "404 Not Found",
            json_response(service_name, "not found", None),
        ),
    };

    write_response(&mut stream, status, "application/json", &body)
}

fn shared_snapshot(service_name: &str, shared_dir: &Path) -> std::io::Result<String> {
    let mut names: Vec<String> = fs::read_dir(shared_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_type()
                .map(|file_type| file_type.is_file())
                .unwrap_or(false)
        })
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();

    let mut items = Vec::new();
    for name in names {
        let content = fs::read_to_string(shared_dir.join(&name))?;
        items.push(format!(
            "\"{}\":\"{}\"",
            escape_json(&name),
            escape_json(content.trim())
        ));
    }

    Ok(format!(
        "{{\"service\":\"{}\",\"files\":{{{}}}}}",
        escape_json(service_name),
        items.join(",")
    ))
}

fn json_response(service_name: &str, status: &str, details: Option<&str>) -> String {
    match details {
        Some(message) => format!(
            "{{\"service\":\"{}\",\"status\":\"{}\",\"details\":\"{}\"}}",
            escape_json(service_name),
            escape_json(status),
            escape_json(message)
        ),
        None => format!(
            "{{\"service\":\"{}\",\"status\":\"{}\"}}",
            escape_json(service_name),
            escape_json(status)
        ),
    }
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn write_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &str,
) -> std::io::Result<()> {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        let unique = format!(
            "lr11-{name}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time went backwards")
                .as_nanos()
        );
        dir.push(unique);
        dir
    }

    #[test]
    fn env_or_trims_and_falls_back() {
        let key = "LR11_RUST_ENV";
        std::env::remove_var(key);
        assert_eq!(env_or(key, "fallback"), "fallback");

        std::env::set_var(key, "  value  ");
        assert_eq!(env_or(key, "fallback"), "value");
        std::env::remove_var(key);
    }

    #[test]
    fn json_helpers_escape_values() {
        assert_eq!(escape_json("a\"b\\c\n"), "a\\\"b\\\\c\\n");
        let payload = json_response("rust-service", "ok", Some("hello"));
        assert!(payload.contains("\"service\":\"rust-service\""));
        assert!(payload.contains("\"status\":\"ok\""));
        assert!(payload.contains("\"details\":\"hello\""));
    }

    #[test]
    fn bootstrap_and_snapshot_shared_data() {
        let dir = temp_dir("shared");
        fs::create_dir_all(&dir).expect("create temp dir");

        bootstrap(&dir, "rust-service").expect("bootstrap shared dir");
        fs::write(dir.join("alpha.txt"), "alpha").expect("write secondary file");

        let snapshot = shared_snapshot("rust-service", &dir).expect("snapshot shared dir");
        assert!(snapshot.contains("\"service\":\"rust-service\""));
        assert!(snapshot.contains("\"alpha.txt\":\"alpha\""));
        assert!(snapshot.contains("\"rust-service.txt\""));

        fs::remove_dir_all(&dir).expect("cleanup temp dir");
    }
}
