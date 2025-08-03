# Zenthos OTA Server

Zenthos OTA Server is a fast, secure, and extensible Over-The-Air (OTA) firmware distribution service built in Rust using Axum. It is designed to support remote firmware updates for embedded devices such as the ESP32 family.

## 🚀 Features

- 🔒 Secure OTA firmware delivery over HTTPS
- 🔍 Device-targeted firmware versioning
- ⚙️ RESTful API for firmware management
- 📊 OpenTelemetry metrics
- 🧠 Designed for integration with CI/CD pipelines and GitHub Actions
- 🛠️ Built with modular architecture for testability and maintainability

## 📚 API Routes

| Method | Path                                 | Description                                 | Auth | Response              |
| ------ | ------------------------------------ | ------------------------------------------- | ---- | --------------------- |
| GET    | `/firmware/{device_model}/{version}` | Download firmware for the specified device  | ❌    | Binary blob           |
| GET    | `/firmware/{device_model}/latest`    | Download the latest firmware for the device | ❌    | Binary blob           |
| GET    | `/health`                            | Health check for uptime monitoring          | ❌    | `200 OK` plain text   |
| GET    | `/docs`                              | OpenApi Swagger UI documentation            | ❌    | `200 OK` HTML content |

## 📈 Telemetry & Observability

Supports:

- OpenTelemetry (traces, metrics, logs)
- Optional integration with Tempo, Loki, and Grafana

Made with ❤️ by @adrianvillanueva997 for the Zenthos platform.
