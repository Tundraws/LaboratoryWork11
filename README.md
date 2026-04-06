# Лабораторная работа №11

## Студент

- ФИО: Мельникова Анастасия Сергеевна
- Группа: 220032-11
- Вариант: 3

## Технологии

- Go
- Python (FastAPI)
- Rust
- Docker / Docker Compose
- GitHub Actions
- HTTP / JSON

## Выполненные задания

### Средней сложности

3. Написать Dockerfile для Rust-приложения.

5. Создать docker-compose.yml, поднимающий Python, Go и Rust сервисы.

7. Использовать volume для обмена данными между контейнерами.

### Повышенной сложности

3. Настроить CI/CD, который собирает и пушит образы для всех трёх языков.

5. Оптимизировать слои Docker-образов для максимального кэширования.

## Реализация

Проект состоит из трёх сервисов:

- Go-сервис отдаёт `/health`, `/shared` и `/write`, записывая данные в общий volume.
- Python-сервис делает то же самое через FastAPI.
- Rust-сервис реализован без внешних web-фреймворков, чтобы Dockerfile оставался компактным и наглядным.

Все три контейнера подключены к одной сети и используют общий volume `lab-data` для обмена файлами.

## Структура проекта

- `go-service/` - Go-сервис с многоэтапной сборкой и оптимизацией кэша.
- `python-service/` - Python/FastAPI-сервис с минимальным набором зависимостей.
- `rust-service/` - Rust-сервис с компактным Dockerfile и общим volume.
- `docker-compose.yml` - совместный запуск трёх сервисов.
- `.github/workflows/docker-images.yml` - CI/CD для сборки и публикации образов.
- `PROMPT_LOG.md` - журнал работы с ИИ.

## Локальный запуск

### Через Docker Compose

```powershell
docker compose up --build
```

После запуска будут доступны:

- Go-сервис: `http://localhost:8080`
- Python-сервис: `http://localhost:8000`
- Rust-сервис: `http://localhost:9000`

### Проверка volume

Каждый сервис пишет служебный файл в общий volume `/shared`. После запуска можно открыть:

- `http://localhost:8080/shared`
- `http://localhost:8000/shared`
- `http://localhost:9000/shared`

## Примеры запросов

### Go

```http
POST /write

GET /shared
```

### Python

```http
POST /write

GET /shared
```

### Rust

```http
POST /write

GET /shared
```

## CI/CD

Workflow в `.github/workflows/docker-images.yml` собирает образы для Go, Python и Rust на каждый push в основную ветку и публикует их в GitHub Container Registry.
