package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"
)

const defaultPort = "8080"

func main() {
	serviceName := envOr("SERVICE_NAME", "go-service")
	sharedDir := envOr("SHARED_DIR", "/shared")
	port := envOr("GO_SERVICE_PORT", defaultPort)

	if err := bootstrap(sharedDir, serviceName); err != nil {
		log.Printf("bootstrap shared data: %v", err)
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		writeJSON(w, http.StatusOK, map[string]string{
			"service": serviceName,
			"status":  "ok",
		})
	})

	mux.HandleFunc("/write", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		if err := appendRecord(sharedDir, serviceName, "manual write"); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}

		writeJSON(w, http.StatusCreated, map[string]string{
			"service": serviceName,
			"status":  "written",
		})
	})

	mux.HandleFunc("/shared", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		files, err := snapshot(sharedDir)
		if err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}

		writeJSON(w, http.StatusOK, map[string]any{
			"service": serviceName,
			"files":   files,
		})
	})

	server := &http.Server{
		Addr:         ":" + port,
		Handler:      mux,
		ReadTimeout:  5 * time.Second,
		WriteTimeout: 5 * time.Second,
		IdleTimeout:  10 * time.Second,
	}

	log.Printf("%s listening on %s", serviceName, server.Addr)
	if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
		log.Fatal(err)
	}
}

func envOr(name, fallback string) string {
	if value := strings.TrimSpace(os.Getenv(name)); value != "" {
		return value
	}
	return fallback
}

func bootstrap(sharedDir, serviceName string) error {
	if err := os.MkdirAll(sharedDir, 0o755); err != nil {
		return err
	}

	return appendRecord(sharedDir, serviceName, "boot")
}

func appendRecord(sharedDir, serviceName, marker string) error {
	path := filepath.Join(sharedDir, serviceName+".txt")
	file, err := os.OpenFile(path, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0o644)
	if err != nil {
		return err
	}
	defer file.Close()

	_, err = fmt.Fprintf(file, "%s | %s | %s\n", time.Now().UTC().Format(time.RFC3339), serviceName, marker)
	return err
}

func snapshot(sharedDir string) (map[string]string, error) {
	entries, err := os.ReadDir(sharedDir)
	if err != nil {
		return nil, err
	}

	names := make([]string, 0, len(entries))
	for _, entry := range entries {
		if entry.Type().IsRegular() {
			names = append(names, entry.Name())
		}
	}
	sort.Strings(names)

	result := make(map[string]string, len(names))
	for _, name := range names {
		content, err := os.ReadFile(filepath.Join(sharedDir, name))
		if err != nil {
			return nil, err
		}
		result[name] = strings.TrimSpace(string(content))
	}

	return result, nil
}

func writeJSON(w http.ResponseWriter, status int, payload any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)

	encoder := json.NewEncoder(w)
	encoder.SetIndent("", "  ")
	_ = encoder.Encode(payload)
}
