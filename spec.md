# Zenthos — Architecture Specification

**Version:** 1.0
**Date:** October 2025
**Status:** Design Phase

---

## Table of Contents

1. [Overview](#overview)
2. [Design Philosophy](#design-philosophy)
3. [System Architecture](#system-architecture)
4. [Operational vs Analytical Planes](#operational-vs-analytical-planes)
5. [Data Flow](#data-flow)
6. [Technology Stack](#technology-stack)
7. [Database Design](#database-design)
8. [Message Protocols](#message-protocols)
9. [Observability Strategy](#observability-strategy)
10. [Security Model](#security-model)
11. [ETL Pipeline](#etl-pipeline)
12. [MLOps Integration](#mlops-integration)
13. [Scalability Limits](#scalability-limits)
14. [Operational Procedures](#operational-procedures)
15. [Development Roadmap](#development-roadmap)

---

## 1. Overview

**Zenthos** is an open-source IoT platform designed for agricultural sensor networks. It provides:

- **Secure OTA firmware updates** for ESP32 devices
- **Real-time telemetry ingestion** via MQTT
- **Dual-plane architecture** separating operational and analytical workloads
- **Observability-first design** using Rust's tracing ecosystem
- **MLOps support** for edge ML deployment (TinyML)

**Target deployment:** Single VPS (8GB RAM, 4-8 vCPUs)
**Target scale:** 1,000-10,000 devices
**Primary language:** Rust

---

## 2. Design Philosophy

### Core Principles

1. **Separation of Concerns:** Operational (real-time) and analytical (historical) planes are distinct
2. **Observability First:** Every service instrumented with `tracing`, not an afterthought
3. **Self-Hosting Friendly:** Single docker-compose file, no Kubernetes required
4. **Learning Platform:** Demonstrates production patterns without over-engineering
5. **Immutable Raw Data:** Never lose the source of truth

### Key Tradeoffs

| Decision | Reasoning | Alternative |
|----------|-----------|-------------|
| Single-node | Self-hosting focus, sufficient for 10k devices | Multi-node cluster |
| TimescaleDB | SQL flexibility, joins with metadata | ClickHouse (higher throughput) |
| Redis Streams | Lighter than RabbitMQ, sufficient for scale | RabbitMQ (more features) |
| Raw Parquet archive | Reprocessing capability, data science friendly | Direct write only (simpler) |
| JWT auth | Simple for MVP, upgrade path to mTLS | mTLS from start (more secure) |

---

## 3. System Architecture

### High-Level Overview

```mermaid
graph TB
    subgraph Devices["🌾 ESP32 Farm Sensors"]
        D[Soil/Weather Sensors<br/>1-10k devices]
    end

    subgraph Ingestion["📡 Ingestion Layer"]
        MQTT[Mosquitto MQTT<br/>TLS + JWT]
        MW[Middleware<br/>Rust + tracing]
    end

    subgraph Operational["💾 Operational Plane<br/>(Real-time, Hot)"]
        PG_OPS[PostgreSQL<br/>Current state<br/>Last 7 days]
        REDIS[Redis<br/>Cache + Queues]
    end

    subgraph Archive["📦 Data Lake<br/>(Raw, Immutable)"]
        MINIO[MinIO<br/>Parquet archives<br/>Bronze/Silver/Gold]
    end

    subgraph Analytical["📊 Analytical Plane<br/>(Historical, Cold)"]
        TS_ANALYTICS[TimescaleDB<br/>Multi-month data<br/>Pre-aggregated]
        ETL[ETL Worker<br/>Nightly batch]
    end

    subgraph Observability["🔭 Observability"]
        OTEL[OTEL Collector]
        TEMPO[Tempo<br/>Traces]
        LOKI[Loki<br/>Logs]
    end

    subgraph Services["🦀 Services"]
        OTA[OTA Server<br/>Firmware delivery]
        WORKER[Workers<br/>Alerts, Jobs]
    end

    subgraph Visualization["📈 Visualization"]
        GRAF[Grafana<br/>Unified dashboards]
    end

    D -->|Telemetry<br/>1-2 min| MQTT
    MQTT --> MW
    MW -->|Write hot data| PG_OPS
    MW -->|Archive raw| MINIO
    MW -->|Async jobs| REDIS
    MW -.->|Traces/Logs| OTEL

    REDIS --> WORKER
    WORKER --> PG_OPS
    WORKER -.->|Traces/Logs| OTEL

    MINIO -->|Nightly ETL| ETL
    ETL -->|Load processed| TS_ANALYTICS

    OTA -->|Serve firmware| MINIO
    OTA -.->|Traces/Logs| OTEL

    OTEL --> TEMPO
    OTEL --> LOKI

    GRAF -->|Query ops data| PG_OPS
    GRAF -->|Query analytics| TS_ANALYTICS
    GRAF -->|Query traces| TEMPO
    GRAF -->|Query logs| LOKI
```

---

## 4. Operational vs Analytical Planes

### The Two-Plane Architecture

```mermaid
graph LR
    subgraph Operational["⚡ OPERATIONAL PLANE"]
        direction TB
        OP1[Real-time queries]
        OP2[Device state]
        OP3[Last 7 days hot data]
        OP4[Sub-100ms latency]
        OP5[High write throughput]
    end

    subgraph Analytical["📊 ANALYTICAL PLANE"]
        direction TB
        AN1[Historical analysis]
        AN2[Trend detection]
        AN3[Months/years of data]
        AN4[Seconds latency OK]
        AN5[Complex aggregations]
    end

    subgraph Bridge["🔄 ETL BRIDGE"]
        direction TB
        BR1[Nightly batch jobs]
        BR2[Data cleaning]
        BR3[Feature engineering]
    end

    Operational -->|Raw archive| Bridge
    Bridge -->|Processed data| Analytical
```

### Operational Plane

**Purpose:** Support real-time device operations and monitoring

**Components:**

- PostgreSQL (operational database)
- Redis (cache, rate limits, job queues)
- Middleware (MQTT ingestion)
- Workers (alert processing)

**Data:**

- Current device state (online/offline, battery level)
- Last 7 days of telemetry (hot data)
- Active alerts and jobs

**Query patterns:**

- Point lookups: "What's the current soil moisture for device X?"
- Recent history: "Show last 24 hours of readings"
- Device management: "Which devices are offline?"

**Performance targets:**

- Write latency: <50ms p99
- Query latency: <100ms p99
- Throughput: 5,000-10,000 msg/sec

### Analytical Plane

**Purpose:** Historical analysis, ML training, business intelligence

**Components:**

- TimescaleDB (analytical database)
- MinIO (data lake - Parquet files)
- ETL workers (batch processing)

**Data:**

- Historical telemetry (months/years)
- Pre-aggregated rollups (hourly, daily)
- ML feature store

**Query patterns:**

- Aggregations: "Average soil moisture by farm for last 3 months"
- Trends: "How has temperature changed year-over-year?"
- ML training: "Fetch feature data for irrigation model"

**Performance targets:**

- Query latency: 1-10 seconds (acceptable)
- ETL runtime: <1 hour for daily batch
- Storage compression: 10:1 ratio with Parquet

---

## 5. Data Flow

### Real-Time Path (Operational)

```mermaid
sequenceDiagram
    participant ESP32
    participant MQTT
    participant Middleware
    participant PostgreSQL
    participant MinIO
    participant Redis

    ESP32->>MQTT: Publish telemetry (1-2 min interval)
    MQTT->>Middleware: Subscribe & receive

    Note over Middleware: #[instrument]<br/>Span: mqtt.handle_message

    Middleware->>Middleware: Parse & validate JSON
    Middleware->>PostgreSQL: Update device.last_seen
    Middleware->>PostgreSQL: INSERT INTO telemetry_hot

    Note over PostgreSQL: Write <50ms

    Middleware->>Redis: XADD raw_archive_stream

    Note over Redis: Async, non-blocking

    Middleware-->>ESP32: ACK (QoS 1)
```

### Batch ETL Path (Analytical)

```mermaid
sequenceDiagram
    participant Scheduler
    participant ETL Worker
    participant MinIO
    participant TimescaleDB

    Note over Scheduler: Daily at 2:00 AM

    Scheduler->>ETL Worker: Trigger daily job

    ETL Worker->>MinIO: Read yesterday's Parquet files

    Note over ETL Worker: Transform with Polars:<br/>- Deduplicate<br/>- Fill nulls<br/>- Aggregate<br/>- Generate features

    ETL Worker->>TimescaleDB: Bulk INSERT cleaned data
    ETL Worker->>TimescaleDB: UPDATE daily aggregates
    ETL Worker->>TimescaleDB: INSERT ml_features

    ETL Worker->>MinIO: Write processed (Silver layer)

    Note over MinIO: Bronze → Silver → Gold<br/>Medallion architecture
```

### Archive Worker (Continuous)

```mermaid
graph LR
    A[Middleware writes to<br/>Redis Stream] -->|Every hour| B[Archive Worker]
    B --> C[Batch 1000 messages]
    C --> D[Convert to Parquet]
    D --> E[Upload to MinIO<br/>/raw/YYYY/MM/DD/HH.parquet]
    E --> F[ACK Redis Stream]
```

---

## 6. Technology Stack

### Core Services

| Component | Technology | Purpose | RAM Usage |
|-----------|-----------|---------|-----------|
| Operational DB | PostgreSQL 16 | Device state, hot telemetry | 2GB |
| Analytical DB | TimescaleDB (Postgres extension) | Historical data, aggregates | 2GB |
| Message Broker | Mosquitto MQTT | Device communication | 200MB |
| Cache/Queue | Redis 7 | Caching, job queues, streams | 500MB |
| Object Storage | MinIO | Firmware, raw archives | 500MB |
| Observability | OTEL Collector | Trace/log collection | 200MB |
| Trace Storage | Grafana Tempo | Distributed tracing | 500MB |
| Log Storage | Grafana Loki | Structured logs | 500MB |
| Visualization | Grafana | Dashboards | 300MB |

**Total: ~6-7GB RAM**

### Application Services (Rust)

| Service | Framework | Responsibility |
|---------|-----------|----------------|
| Middleware | Tokio + rumqttc | MQTT ingestion, validation, DB writes |
| OTA Server | Axum | Firmware metadata API, download proxy |
| Archive Worker | Tokio | Batch raw messages to Parquet |
| ETL Worker | Tokio + Polars | Nightly transformation pipeline |
| Alert Worker | Tokio | Process alert rules, notifications |

### Key Libraries

**Rust:**

- `tracing` + `tracing-opentelemetry` — Instrumentation
- `sqlx` — Database access (async, compile-time checked)
- `rumqttc` — MQTT client
- `axum` — HTTP server
- `serde` + `serde_json` — Serialization
- `polars` — DataFrame processing (Rust alternative to Pandas)
- `arrow` / `parquet` — Columnar data format

**Infrastructure:**

- Docker Compose — Local development
- Traefik — Reverse proxy, TLS termination
- Let's Encrypt — Automatic HTTPS certificates

---

## 7. Database Design

### Operational Database Schema

```sql
-- Database: zenthos_ops

-- Current device state
CREATE TABLE devices (
  id TEXT PRIMARY KEY,              -- e.g., "farm-alpha-sensor-042"
  farm_id UUID NOT NULL,
  device_type TEXT NOT NULL,        -- soil_sensor, weather_station
  location GEOGRAPHY(POINT),        -- PostGIS for spatial queries
  installed_at TIMESTAMPTZ,
  last_seen TIMESTAMPTZ,            -- Updated on every message
  firmware_version TEXT,
  battery_voltage DOUBLE PRECISION,
  status TEXT DEFAULT 'unknown',    -- online, offline, maintenance
  metadata JSONB
);

-- Hot telemetry (last 7 days only)
CREATE TABLE telemetry_hot (
  time TIMESTAMPTZ NOT NULL,
  device_id TEXT NOT NULL,
  farm_id UUID NOT NULL,

  -- Sensor readings
  soil_moisture DOUBLE PRECISION,
  temperature DOUBLE PRECISION,
  humidity DOUBLE PRECISION,
  light_level DOUBLE PRECISION,
  battery_voltage DOUBLE PRECISION,
  signal_strength INT,

  -- Metadata
  firmware_version TEXT,
  message_id UUID UNIQUE NOT NULL   -- Idempotency key
);

SELECT create_hypertable('telemetry_hot', 'time');
SELECT add_retention_policy('telemetry_hot', INTERVAL '7 days');

-- Indexes for fast lookups
CREATE INDEX idx_telemetry_hot_device ON telemetry_hot (device_id, time DESC);
CREATE INDEX idx_telemetry_hot_farm ON telemetry_hot (farm_id, time DESC);

-- Active alerts
CREATE TABLE alerts (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  created_at TIMESTAMPTZ DEFAULT now(),
  device_id TEXT REFERENCES devices(id),
  farm_id UUID,
  alert_type TEXT,        -- low_battery, high_temp, offline
  severity TEXT,          -- info, warning, critical
  message TEXT,
  acknowledged BOOLEAN DEFAULT false,
  resolved_at TIMESTAMPTZ
);

-- OTA firmware releases
CREATE TABLE firmware_releases (
  device_model TEXT NOT NULL,
  version TEXT NOT NULL,
  binary_url TEXT NOT NULL,
  sha256 TEXT NOT NULL,
  signature TEXT NOT NULL,        -- ED25519 signature
  release_notes TEXT,
  uploaded_at TIMESTAMPTZ DEFAULT now(),
  PRIMARY KEY (device_model, version)
);

CREATE INDEX idx_firmware_latest ON firmware_releases (device_model, uploaded_at DESC);
```

### Analytical Database Schema

```sql
-- Database: zenthos_analytics

-- Historical telemetry (multi-month storage)
CREATE TABLE telemetry_historical (
  time TIMESTAMPTZ NOT NULL,
  device_id TEXT NOT NULL,
  farm_id UUID NOT NULL,

  soil_moisture DOUBLE PRECISION,
  temperature DOUBLE PRECISION,
  humidity DOUBLE PRECISION,
  light_level DOUBLE PRECISION,
  battery_voltage DOUBLE PRECISION,

  -- Enriched during ETL
  weather_condition TEXT,
  irrigation_event BOOLEAN,

  message_id UUID UNIQUE NOT NULL
);

SELECT create_hypertable('telemetry_historical', 'time');

-- Continuous aggregates for fast queries
CREATE MATERIALIZED VIEW telemetry_hourly
WITH (timescaledb.continuous) AS
SELECT
  time_bucket('1 hour', time) AS bucket,
  device_id,
  farm_id,
  AVG(soil_moisture) as soil_avg,
  AVG(temperature) as temp_avg,
  MAX(temperature) as temp_max,
  MIN(temperature) as temp_min,
  STDDEV(temperature) as temp_stddev,
  COUNT(*) as sample_count
FROM telemetry_historical
GROUP BY bucket, device_id, farm_id;

SELECT add_continuous_aggregate_policy('telemetry_hourly',
  start_offset => INTERVAL '3 hours',
  end_offset => INTERVAL '1 hour',
  schedule_interval => INTERVAL '1 hour');

CREATE MATERIALIZED VIEW telemetry_daily
WITH (timescaledb.continuous) AS
SELECT
  time_bucket('1 day', time) AS day,
  device_id,
  farm_id,
  AVG(soil_moisture) as soil_avg,
  AVG(temperature) as temp_avg,
  MAX(temperature) as temp_max,
  MIN(temperature) as temp_min,
  COUNT(*) as sample_count
FROM telemetry_historical
GROUP BY day, device_id, farm_id;

-- ML feature store
CREATE TABLE ml_features (
  device_id TEXT NOT NULL,
  date DATE NOT NULL,

  -- Rolling averages
  soil_7d_avg DOUBLE PRECISION,
  soil_7d_stddev DOUBLE PRECISION,
  temp_7d_avg DOUBLE PRECISION,

  -- Trends
  soil_trend_slope DOUBLE PRECISION,      -- Linear regression slope
  temp_variability DOUBLE PRECISION,

  -- Context
  days_since_irrigation INT,
  days_since_rain INT,

  PRIMARY KEY (device_id, date)
);
```

### Farm Context Schema

```sql
-- Shared across both databases

CREATE TABLE farms (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  owner TEXT NOT NULL,
  location GEOGRAPHY(POINT),
  total_area_hectares DOUBLE PRECISION,
  crop_type TEXT,
  created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE irrigation_events (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  farm_id UUID REFERENCES farms(id),
  timestamp TIMESTAMPTZ NOT NULL,
  duration_minutes INT,
  water_volume_liters DOUBLE PRECISION,
  triggered_by TEXT        -- manual, scheduled, ml_model
);
```

---

## 8. Message Protocols

### MQTT Topic Structure

```
zenthos/{device_model}/{device_id}/{message_type}

Examples:
  zenthos/esp32-s3/farm-alpha-sensor-042/telemetry
  zenthos/esp32-s3/farm-alpha-sensor-042/status
  zenthos/esp32-cam/farm-beta-cam-001/inference
```

### Telemetry Message Format

```json
{
  "message_id": "550e8400-e29b-41d4-a716-446655440000",
  "device_id": "farm-alpha-sensor-042",
  "timestamp": 1728561234,
  "sensors": {
    "soil_moisture": 42.5,
    "temperature": 22.3,
    "humidity": 65.8,
    "light_level": 780,
    "battery_voltage": 3.85
  },
  "firmware_version": "v0.2.1",
  "signal_strength": -67
}
```

**Field specifications:**

- `message_id`: UUIDv4, for idempotency and deduplication
- `device_id`: Unique identifier, matches device registration
- `timestamp`: Unix epoch seconds (UTC)
- `sensors`: Flexible object, varies by device type
- `firmware_version`: Semantic versioning
- `signal_strength`: WiFi RSSI in dBm

### Status Message Format

```json
{
  "message_id": "...",
  "device_id": "farm-alpha-sensor-042",
  "timestamp": 1728561234,
  "status": "online",
  "uptime_seconds": 86400,
  "free_heap_bytes": 45000,
  "boot_count": 12
}
```

### OTA Response Format

```json
{
  "device_model": "esp32-s3",
  "version": "v0.2.1",
  "binary_url": "https://ota.zenthos.example/firmware/esp32-s3/v0.2.1.bin",
  "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "signature": "base64-encoded-ed25519-signature",
  "size_bytes": 1048576,
  "uploaded_at": "2025-10-11T10:00:00Z",
  "required": false,
  "rollout_percentage": 10
}
```

---

## 9. Observability Strategy

### The Two Telemetry Types

**1. Device Telemetry** (Business Data)

- Sensor readings from ESP32 devices
- Stored in TimescaleDB
- Queried for farming insights
- Long-term retention

**2. Application Traces** (Operational Data)

- Rust service instrumentation
- Collected via OpenTelemetry
- Stored in Tempo/Loki
- Short-term retention (7 days)

### Rust Tracing Implementation

**Every service exports:**

- Traces (spans with timing)
- Structured logs (JSON with trace_id)
- Metrics (counters, histograms)

**Instrumentation pattern:**

```
#[instrument] macro on async functions
  → Automatic span creation
  → Child spans for sub-operations
  → Trace context propagation
  → Log correlation via trace_id
```

### Trace Flow

```mermaid
graph LR
    A[Middleware<br/>] -->|OTLP gRPC| B[OTEL Collector]
    C[OTA Server<br/>] -->|OTLP gRPC| B
    D[Workers<br/>] -->|OTLP gRPC| B

    B -->|Export traces| E[Tempo]
    B -->|Export logs| F[Loki]

    E --> G[Grafana]
    F --> G

    H[TimescaleDB<br/>Device telemetry] --> G
```

### Key Metrics

**Operational metrics:**

- `zenthos.middleware.messages_received` (counter)
- `zenthos.middleware.db_write_latency_ms` (histogram)
- `zenthos.middleware.validation_errors` (counter)
- `zenthos.ota.firmware_downloads` (counter)
- `zenthos.devices.online_count` (gauge)

**Business metrics (stored in PostgreSQL):**

- Device online/offline counts by farm
- Average soil moisture by farm
- Low battery device count
- Failed message count

### Grafana Dashboard Structure

```
Dashboard: Fleet Overview
  - Device count by status (online/offline/maintenance)
  - Firmware version distribution
  - Geographic map of device locations
  - Alert summary

Dashboard: Farm Analytics
  - Soil moisture trends by farm
  - Temperature/humidity charts
  - Irrigation event timeline
  - ML model predictions

Dashboard: System Health
  - Middleware throughput (msg/sec)
  - Database write latency
  - ETL job status
  - Disk usage trends

Dashboard: Distributed Traces
  - Request traces (mqtt.handle_message spans)
  - Latency heatmaps
  - Error rate by service
  - Slow query identification
```

---

## 10. Security Model

### Authentication

**Device Authentication (MQTT):**

- **MVP:** Username/password stored in Redis (fast lookup)
- **Format:** `device_id:jwt_token`
- **Rotation:** 30-day expiry, automatic renewal
- **Future:** Upgrade to mTLS client certificates

**User Authentication (APIs):**

- JWT tokens issued via login endpoint
- Refresh token pattern (7-day access, 30-day refresh)
- Scopes: `admin`, `operator`, `viewer`

**Service-to-Service:**

- Shared secrets via environment variables
- MinIO access keys
- PostgreSQL connection strings

### Authorization

**Role-based access control:**

| Role | Permissions |
|------|-------------|
| admin | Full access, firmware publish, user management |
| operator | View dashboards, trigger manual OTA, acknowledge alerts |
| viewer | Read-only dashboard access |
| device | Publish telemetry (scoped to device_id), fetch OTA |

### Network Security

**TLS everywhere:**

- MQTT: Port 8883 (TLS), certificate validation
- HTTPS: Let's Encrypt certificates via Traefik
- Internal services: Docker bridge network (no public access)

**Firewall rules:**

```
Public ports:
  - 443 (HTTPS for OTA, Grafana)
  - 8883 (MQTTS for devices)
  - 22 (SSH for admin, key-based only)

Internal only:
  - 5432 (PostgreSQL)
  - 6379 (Redis)
  - 9000 (MinIO S3 API)
  - 1883 (MQTT non-TLS, dev only)
```

### Firmware Signing

**ED25519 signature verification:**

1. CI builds firmware, signs with private key
2. Public key embedded in device firmware at compile time
3. Device downloads firmware + signature
4. Device verifies signature before applying
5. Rollback to previous version if verification fails

**Key management:**

- Private key stored in GitHub Secrets (CI only)
- Public key in device firmware source
- Key rotation procedure documented

### Rate Limiting

**Device rate limits (enforced by middleware):**

- 1 message per second per device
- 100 OTA requests per hour per device
- Connection throttling: 1000 new connections per minute

**API rate limits (enforced by Traefik):**

- 100 requests per minute per IP for public endpoints
- 1000 requests per minute for authenticated users

---

## 11. ETL Pipeline

### Pipeline Schedule

**Hourly jobs:**

- Archive raw telemetry to Parquet (MinIO)
- Aggregate alerts for summary dashboard

**Daily jobs (2:00 AM):**

- Load previous day's data to analytical DB
- Compute daily aggregates
- Generate ML features
- Cleanup old operational data

**Weekly jobs (Sunday 3:00 AM):**

- Recompute long-term trends
- Database vacuum and analyze
- Backup verification test

### Data Lake Structure (Medallion Architecture)

```
MinIO buckets:

zenthos-bronze/          # Raw, immutable
├─ telemetry/
│  └─ 2025/10/11/
│     ├─ 00.parquet    # Hourly batches, exactly as received
│     ├─ 01.parquet
│     └─ ...

zenthos-silver/          # Cleaned, validated
├─ telemetry_cleaned/
│  └─ 2025/10/
│     └─ data.parquet  # Daily partitions, deduplicated

zenthos-gold/            # Business-ready
├─ telemetry_daily/
│  └─ 2025/10.parquet  # Pre-aggregated for dashboards
├─ ml_features/
│  └─ 2025-10.parquet  # Feature store for ML training

zenthos-firmware/        # Application artifacts
├─ esp32-s3/
│  └─ v0.2.1.bin
└─ models/
   └─ irrigation-model.tflite
```

### ETL Transformation Steps

**Bronze → Silver:**

1. Deduplicate by message_id
2. Validate schema and data types
3. Drop malformed records (log to error table)
4. Fill missing timestamps (use ingest_ts)
5. Enrich with device metadata (farm_id, location)
6. Write to Silver layer

**Silver → Gold:**

1. Compute hourly/daily aggregates
2. Generate rolling window features (7-day avg, trends)
3. Join with irrigation events
4. Create ML feature vectors
5. Write to Gold layer + analytical DB

### Idempotency Strategy

**Ensure ETL can be safely re-run:**

- Use `INSERT ... ON CONFLICT DO UPDATE` for upserts
- Track processed files in `etl_runs` table
- Compare checksum before reprocessing
- Allow manual backfill with date range parameter

---

## 12. MLOps Integration

### Phase 1: Infrastructure (Current Scope)

**What's included now:**

- Message format includes `model_version` field
- Storage for TFLite models in MinIO
- OTA server can deliver models alongside firmware

**Not yet implemented:**

- Model training pipeline
- Feature store integration
- Inference result tracking

### Phase 2: Edge ML Pipeline (Future)

```mermaid
graph TB
    A[Historical Data<br/>TimescaleDB] -->|Extract features| B[Jupyter Notebook<br/>Model training]
    B -->|Train/evaluate| C[MLflow Registry]
    C -->|Export .pb| D[TFLite Converter]
    D -->|Quantize INT8| E[Model .tflite]
    E -->|Upload| F[MinIO models/]
    F -->|Publish metadata| G[OTA Server]
    G -->|Deliver to devices| H[ESP32 inference]
    H -->|Results via MQTT| I[TimescaleDB]
    I -->|Feedback loop| B
```

**Example use case: Irrigation prediction model**

- Input features: 7-day soil moisture avg, temperature trend, days since last irrigation
- Output: Binary classification (irrigate / don't irrigate)
- Model: Quantized decision tree or small neural network
- Size: <100KB TFLite model
- Inference: On-device every hour

### Model Versioning

```json
{
  "model_name": "irrigation-predictor",
  "version": "m-2025-10-11-abc123",
  "target_devices": ["esp32-s3"],
  "url": "https://ota.zenthos.example/models/irrigation-predictor-v1.tflite",
  "sha256": "...",
  "size_bytes": 87040,
  "input_shape": [1, 7],
  "output_classes": ["irrigate", "skip"]
}
```

---

## 13. Scalability Limits

### Single-Node Capacity

| Metric | Comfortable | Maximum | Bottleneck |
|--------|-------------|---------|------------|
| Active devices | 1,000 | 10,000 | MQTT connections |
| Messages/sec | 100 | 1,000 | Middleware + DB writes |
| Telemetry frequency | 1-2 min | 30 sec | Network + storage |
| Hot data retention | 7 days | 14 days | PostgreSQL size |
| Cold data retention | 90 days | 365 days | MinIO disk |

### Performance Benchmarks

**Target performance (single VPS):**

- MQTT ingestion: 1,000 msg/sec sustained
- Database writes: 5,000 inserts/sec (batched)
- OTA downloads: 100 concurrent (CDN-backed)
- Dashboard queries: <2 sec p95
- ETL processing: <1 hour for 24-hour batch

### Scale-Out Triggers

**When to move beyond single-node:**

1. Device count exceeds 10,000
2. Message rate exceeds 1,000/sec consistently
3. Database size exceeds 500GB
4. ETL jobs take >2 hours
5. Query latency degrades despite optimization

**Scale-out path:**

- Kubernetes deployment (horizontal scaling)
- Managed TimescaleDB or ClickHouse
- Managed MQTT broker (EMQX cloud)
- Read replicas for analytical queries
- CDN for firmware distribution

---

## 14. Operational Procedures

### Deployment

**Initial setup:**

```bash
1. Provision VPS (Ubuntu 22.04, 8GB RAM, 100GB SSD)
2. Install Docker + Docker Compose
3. Clone repository
4. Configure .env (passwords, domain, etc.)
5. Run: docker-compose up -d
6. Verify health checks
7. Configure DNS (A records for ota.domain, grafana.domain)
8. Traefik automatically provisions Let's Encrypt certs
```

**Update procedure:**

```bash
1. git pull
2. docker-compose pull
3. docker-compose up -d --remove-orphans
4. Verify health endpoints
5. Check Grafana dashboards
```

**Rollback procedure:**

```bash
1. git checkout <previous-commit>
2. docker-compose up -d
3. Restore database from backup if needed
```

### Backup Strategy

**PostgreSQL (both operational and analytical):**

- Continuous WAL archiving to MinIO
- Daily full backup via pg_basebackup
- Retention: 7 daily, 4 weekly, 12 monthly
- Point-in-time recovery capability
- RPO: 5 minutes, RTO: 15 minutes

**MinIO (firmware and raw data):**

- Object versioning enabled
- Lifecycle rules: Delete raw data >90 days
- Weekly rsync to external backup (Backblaze B2)
- Firmware artifacts: permanent retention

**Redis:**

- AOF (Append-Only File) persistence
- RDB snapshots every hour
- Acceptable data loss: Last hour of cache (rebuilds from DB)

**Configuration and secrets:**

- Version controlled (encrypted with SOPS)
- Stored in separate backup location
- Include .env templates, docker-compose.yml, Grafana dashboards

**Backup verification:**

- Monthly automated restore test
- Document restore procedure in runbook
- Test recovery time objectives

### Monitoring and Alerting

**Health checks:**

- All services expose `/health` endpoints
- Docker healthcheck configured for auto-restart
- External monitoring: UptimeRobot (5-min checks)

**Critical alerts (sent to email/Slack):**

- Database connection failures
- MQTT broker offline
- Disk usage >85%
- ETL job failures
- >100 devices offline simultaneously
- Certificate expiration <7 days

**Warning alerts:**

- High memory usage (>80%)
- Slow query detected (>5 sec)
- Device offline >1 hour
- Low battery (<3.3V)

**Alert channels:**

- Email for critical alerts
- Slack for warnings
- Grafana built-in alerting
- PagerDuty integration (optional)

### Disaster Recovery Scenarios

**Scenario 1: Database corruption**

```
1. Stop application services
2. Restore from most recent backup
3. Replay WAL logs for point-in-time recovery
4. Verify data integrity
5. Restart services
Expected downtime: 15-30 minutes
```

**Scenario 2: Complete VPS failure**

```
1. Provision new VPS
2. Restore PostgreSQL from backup
3. Restore MinIO from external backup
4. Deploy docker-compose stack
5. Update DNS records
6. Verify device reconnection
Expected downtime: 1-2 hours
```

**Scenario 3: Accidental data deletion**

```
1. Identify timestamp of deletion
2. Restore database to point before deletion
3. Export affected data
4. Merge into current database
5. Verify data consistency
Expected downtime: None (read-only mode during recovery)
```

### Database Maintenance

**Weekly tasks:**

- VACUUM ANALYZE on operational database
- Check index bloat
- Review slow query log
- Verify TimescaleDB compression jobs

**Monthly tasks:**

- Review and optimize continuous aggregates
- Update table statistics
- Check replication lag (if applicable)
- Test backup restoration

**Quarterly tasks:**

- Review and adjust retention policies
- Evaluate partition strategy
- Performance benchmark comparison
- Security audit (dependency updates)

---

## 15. Development Roadmap

### Phase 0: Foundation (Weeks 1-2)

**Goal:** Scaffold and development environment

**Deliverables:**

- Monorepo structure (Cargo workspace)
- docker-compose.yml for dev stack
- PostgreSQL + TimescaleDB schemas
- Basic CI pipeline (lint, test, build)

**Success criteria:**

- `make dev-up` starts all services
- PostgreSQL accessible with seed data
- Basic health checks pass

---

### Phase 1: Operational Plane (Weeks 3-6)

**Goal:** Real-time device telemetry ingestion

**Deliverables:**

- Middleware service (MQTT → PostgreSQL)
- Rust tracing instrumentation
- Device registration API
- Basic Grafana dashboard
- Device simulator for testing

**Success criteria:**

- Simulated devices publish telemetry
- Data visible in PostgreSQL
- Traces visible in Tempo
- Real-time dashboard updates

**Key learnings:**

- Rust async programming (Tokio)
- MQTT protocol and QoS
- tracing crate and OpenTelemetry
- TimescaleDB hypertables

---

### Phase 2: OTA Server (Weeks 7-9)

**Goal:** Firmware delivery infrastructure

**Deliverables:**

- OTA server (Axum HTTP API)
- MinIO integration
- Firmware metadata endpoints
- GitHub Actions for firmware builds
- ED25519 signing implementation

**Success criteria:**

- Device can query `/firmware/latest`
- Firmware binary served from MinIO
- Signature verification works
- CI automatically builds and publishes

**Key learnings:**

- REST API design with Axum
- Object storage (S3-compatible)
- Cryptographic signatures
- CI/CD automation

---

### Phase 3: Analytical Plane (Weeks 10-13)

**Goal:** Historical data and ETL pipeline

**Deliverables:**

- Raw data archival to Parquet
- ETL worker (batch processing)
- Analytical database setup
- Continuous aggregates
- Historical dashboards

**Success criteria:**

- Raw data archived hourly
- Nightly ETL job completes successfully
- Analytical queries return results
- Dashboard shows multi-month trends

**Key learnings:**

- Parquet format and Arrow
- Data lake architecture (Medallion)
- Batch processing patterns
- Polars DataFrame library

---

### Phase 4: Observability & Polish (Weeks 14-16)

**Goal:** Production-ready monitoring and docs

**Deliverables:**

- Complete Grafana dashboard suite
- Alert rules and notifications
- Comprehensive documentation
- Performance benchmarks
- Security hardening

**Success criteria:**

- All critical metrics monitored
- Alerts trigger correctly
- Documentation covers deployment
- Security best practices implemented

**Key learnings:**

- Grafana dashboard design
- Alert rule optimization
- Technical writing
- Security auditing

---

### Phase 5: MLOps (Future, Weeks 17-22)

**Goal:** Edge ML deployment capability

**Deliverables:**

- MLflow integration
- TFLite model converter
- Feature store implementation
- Model versioning in OTA
- Inference result tracking

**Success criteria:**

- Train model on historical data
- Convert and quantize for ESP32
- Deploy via OTA
- Collect inference results

**Key learnings:**

- ML model lifecycle
- TinyML constraints
- Feature engineering
- Model monitoring

---

## 16. API Specifications

### OTA Server Endpoints

**GET /health**

```
Response: 200 OK
{
  "status": "ok",
  "timestamp": 1728561234,
  "version": "0.1.0"
}
```

**GET /firmware/{device_model}/{version}**

```
Path params:
  - device_model: esp32-s3, esp32-cam
  - version: v0.2.1 or "latest"

Headers:
  - Authorization: Bearer <device_jwt>

Response: 200 OK
{
  "device_model": "esp32-s3",
  "version": "v0.2.1",
  "binary_url": "https://ota.zenthos.example/download/...",
  "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "signature": "base64-encoded-signature",
  "size_bytes": 1048576,
  "uploaded_at": "2025-10-11T10:00:00Z",
  "required": false,
  "rollout_percentage": 100
}

Errors:
  - 404: Firmware version not found
  - 401: Invalid or missing authentication
  - 403: Device not eligible for this version
```

**POST /firmware/publish**

```
Headers:
  - Authorization: Bearer <admin_token>

Body:
{
  "device_model": "esp32-s3",
  "version": "v0.2.1",
  "binary_data": "<base64-encoded-binary>",
  "release_notes": "Bug fixes and improvements",
  "rollout_percentage": 10
}

Response: 201 Created
{
  "id": "uuid",
  "status": "published",
  "url": "https://ota.zenthos.example/firmware/esp32-s3/v0.2.1.bin"
}
```

**GET /devices/{device_id}/status**

```
Headers:
  - Authorization: Bearer <operator_token>

Response: 200 OK
{
  "device_id": "farm-alpha-sensor-042",
  "status": "online",
  "last_seen": "2025-10-11T12:34:56Z",
  "firmware_version": "v0.2.1",
  "battery_voltage": 3.85,
  "pending_update": null
}
```

### Management API Endpoints

**POST /devices/register**

```
Headers:
  - Authorization: Bearer <admin_token>

Body:
{
  "device_id": "farm-alpha-sensor-042",
  "device_type": "soil_sensor",
  "farm_id": "uuid",
  "location": {
    "lat": 40.7128,
    "lon": -74.0060
  }
}

Response: 201 Created
{
  "device_id": "farm-alpha-sensor-042",
  "jwt_token": "<device-auth-token>",
  "mqtt_broker": "mqtt.zenthos.example:8883"
}
```

**GET /analytics/farm/{farm_id}/summary**

```
Query params:
  - start_date: 2025-10-01
  - end_date: 2025-10-11

Response: 200 OK
{
  "farm_id": "uuid",
  "device_count": 15,
  "avg_soil_moisture": 42.5,
  "avg_temperature": 22.3,
  "alerts_count": 3,
  "irrigation_events": 5
}
```

---

## 17. Configuration Management

### Environment Variables

**Required for all services:**

```bash
# Database
DATABASE_URL=postgresql://user:pass@postgres:5432/zenthos_ops
ANALYTICS_DATABASE_URL=postgresql://user:pass@postgres:5432/zenthos_analytics

# Redis
REDIS_URL=redis://redis:6379

# MQTT
MQTT_BROKER_URL=mqtt://mosquitto:1883
MQTT_TLS_ENABLED=true
MQTT_CA_CERT_PATH=/certs/ca.crt

# MinIO
MINIO_ENDPOINT=http://minio:9000
MINIO_ACCESS_KEY=minioadmin
MINIO_SECRET_KEY=minioadmin
MINIO_BUCKET_FIRMWARE=zenthos-firmware
MINIO_BUCKET_RAW=zenthos-bronze

# OpenTelemetry
OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317
OTEL_SERVICE_NAME=zenthos-middleware
RUST_LOG=info,zenthos=debug

# Security
JWT_SECRET=<strong-random-secret>
FIRMWARE_SIGNING_PUBLIC_KEY=<ed25519-public-key>

# Feature flags
ENABLE_RAW_ARCHIVAL=true
ENABLE_ML_INFERENCE=false
```

**Service-specific:**

```bash
# Middleware
MQTT_TOPICS=zenthos/+/+/telemetry,zenthos/+/+/status
BATCH_SIZE=100
BATCH_TIMEOUT_MS=5000

# ETL Worker
ETL_SCHEDULE_CRON=0 2 * * *
ETL_BACKFILL_DAYS=1

# OTA Server
OTA_PORT=8080
OTA_ROLLOUT_DEFAULT_PERCENTAGE=10
```

### Secrets Management

**Development:**

- Plain .env file (gitignored)
- docker-compose reads from .env

**Production:**

- SOPS (encrypted .env in git)
- Or HashiCorp Vault
- Or cloud provider secrets (AWS Secrets Manager, etc.)

**Never commit:**

- Database passwords
- JWT secrets
- API keys
- Private signing keys

---

## 18. Testing Strategy

### Unit Tests

**Coverage targets:**

- Business logic: >80%
- Data transformations: 100%
- Parsers and validators: 100%

**Key areas:**

- Message parsing and validation
- Database query builders
- Authentication/authorization logic
- ETL transformation functions

### Integration Tests

**Test environment:**

- Docker Compose test stack
- Isolated databases per test suite
- Testcontainers for Rust integration tests

**Test scenarios:**

- MQTT message flow (publish → DB)
- OTA firmware query and download
- ETL pipeline (raw → processed)
- Alert rule triggering

### End-to-End Tests

**Full system tests:**

- Device simulator publishes telemetry
- Verify data in operational DB
- Trigger ETL job manually
- Verify data in analytical DB
- Query Grafana API for dashboard data

**Performance tests:**

- Simulate 1,000 devices
- Measure throughput and latency
- Identify bottlenecks
- Document results

### Load Testing

**Tools:**

- Rust-based device simulator
- K6 for HTTP endpoint load testing
- MQTT bench for broker stress testing

**Scenarios:**

- Gradual ramp: 0 → 1,000 devices over 10 min
- Sustained load: 1,000 devices @ 1 msg/min for 1 hour
- Spike test: 10,000 messages in 1 minute

**Metrics to track:**

- Message latency (MQTT → DB)
- Database write throughput
- CPU and memory usage
- Error rates

---

## 19. Documentation Structure

### For Developers

```
docs/
├── README.md                    # Project overview
├── ARCHITECTURE.md              # This file
├── CONTRIBUTING.md              # How to contribute
├── getting-started/
│   ├── local-setup.md          # Run locally
│   ├── first-device.md         # Connect test device
│   └── debugging.md            # Common issues
├── architecture/
│   ├── data-flow.md            # Telemetry pipeline
│   ├── observability.md        # Tracing guide
│   ├── security.md             # Auth/authz details
│   └── decisions/              # ADRs
│       ├── 001-rust-language.md
│       ├── 002-timescaledb.md
│       └── 003-two-plane-architecture.md
├── api/
│   ├── ota-server.md           # API reference
│   ├── mqtt-protocol.md        # Message formats
│   └── management-api.md       # Admin endpoints
├── operations/
│   ├── deployment.md           # VPS setup guide
│   ├── monitoring.md           # Grafana dashboards
│   ├── backup-restore.md       # DR procedures
│   └── troubleshooting.md      # Common problems
└── mlops/
    ├── training-pipeline.md    # How to train models
    ├── model-deployment.md     # OTA for models
    └── tinyml-guide.md         # ESP32 constraints
```

### For End Users

```
website/
├── index.md                    # Landing page
├── quick-start.md              # 5-minute setup
├── user-guide/
│   ├── dashboard-tour.md       # Grafana walkthrough
│   ├── device-setup.md         # Flash and configure ESP32
│   └── alerts.md               # Configure alerts
└── tutorials/
    ├── soil-moisture-monitoring.md
    ├── irrigation-automation.md
    └── ml-irrigation-prediction.md
```

---

## 20. Success Metrics

### Technical Metrics

**Performance:**

- P99 message latency: <100ms (operational path)
- Database write throughput: >5,000 inserts/sec
- ETL completion time: <1 hour per day
- Dashboard load time: <2 seconds

**Reliability:**

- Uptime: >99.5% (acceptable ~3.6 hours/month)
- Data loss: <0.01% of messages
- Backup success rate: 100%
- Recovery time: <30 minutes

**Scalability:**

- Support 10,000 devices on single node
- Handle 1,000 messages/sec sustained
- Store 90 days of telemetry efficiently

### Portfolio Metrics

**Engagement:**

- GitHub stars: Target 100+ in first 6 months
- Documentation page views
- Issues and pull requests from community
- Blog post shares and comments

**Learning outcomes:**

- Demonstrated Rust proficiency (async, tracing, production patterns)
- Time-series database expertise (TimescaleDB)
- Data architecture knowledge (OLTP vs OLAP)
- MLOps fundamentals (edge ML deployment)
- DevOps skills (Docker, CI/CD, observability)

---

## 21. Known Limitations and Future Work

### Current Limitations

**Single-node architecture:**

- No high availability or automatic failover
- Downtime during maintenance windows
- Vertical scaling only

**Authentication:**

- Simple JWT, not enterprise SSO
- No multi-tenant isolation
- Manual device provisioning

**MLOps:**

- No automated model training pipeline
- Manual model deployment process
- Limited model monitoring

**Observability:**

- 7-day trace retention (cost constraint)
- No anomaly detection for device behavior
- Manual alert threshold tuning

### Future Enhancements

**Phase 6: Multi-tenancy**

- Row-level security in PostgreSQL
- Tenant-specific dashboards
- Resource quotas and rate limits
- White-label deployments

**Phase 7: Advanced ML**

- Automated retraining pipeline
- A/B testing for models
- Federated learning across devices
- Real-time anomaly detection

**Phase 8: Scale-out**

- Kubernetes deployment manifests
- Horizontal service scaling
- Database sharding strategy
- Multi-region replication

**Phase 9: Edge Computing**

- LoRaWAN support for remote sensors
- Edge gateway aggregation
- Offline-first device operation
- Cellular connectivity (LTE-M/NB-IoT)

---

## 22. Glossary

**Terms:**

- **OLTP:** Online Transaction Processing (operational database)
- **OLAP:** Online Analytical Processing (analytical database)
- **Hypertable:** TimescaleDB's abstraction for time-series tables
- **Continuous Aggregate:** Materialized view that auto-refreshes (TimescaleDB)
- **Medallion Architecture:** Bronze/Silver/Gold data lake pattern
- **TinyML:** Machine learning on microcontrollers
- **OTA:** Over-The-Air (firmware updates)
- **MQTT:** Message Queuing Telemetry Transport (IoT protocol)
- **QoS:** Quality of Service (MQTT reliability levels)
- **OTLP:** OpenTelemetry Protocol (trace/log export format)

**Abbreviations:**

- **ETL:** Extract, Transform, Load
- **RPO:** Recovery Point Objective (acceptable data loss)
- **RTO:** Recovery Time Objective (acceptable downtime)
- **VPS:** Virtual Private Server
- **mTLS:** Mutual TLS (two-way certificate authentication)
- **JWT:** JSON Web Token

---

## 23. References and Resources

### Official Documentation

- TimescaleDB: <https://docs.timescale.com>
- PostgreSQL: <https://www.postgresql.org/docs>
- Rust tracing: <https://docs.rs/tracing>
- OpenTelemetry: <https://opentelemetry.io>
- MQTT: <https://mqtt.org/mqtt-specification>

### Learning Resources

- "Designing Data-Intensive Applications" by Martin Kleppmann
- "Database Internals" by Alex Petrov
- Rust Async Book: <https://rust-lang.github.io/async-book>
- TimescaleDB Best Practices: <https://docs.timescale.com/timescaledb/latest/how-to-guides>

### Similar Projects

- ThingsBoard (open-source IoT platform)
- Mainflux (IoT platform in Go)
- ChirpStack (LoRaWAN network server)

---

## Appendix A: Decision Records

### ADR-001: Use Rust for Backend Services

**Status:** Accepted

**Context:** Need performant, memory-safe language for IoT backend.

**Decision:** Use Rust for all backend services (middleware, OTA, workers).

**Consequences:**

- Pros: Performance, safety, excellent async ecosystem, learning objective
- Cons: Slower development initially, steeper learning curve
- Alternative considered: Go (easier but less learning value)

---

### ADR-002: Separate Operational and Analytical Databases

**Status:** Accepted

**Context:** Different query patterns for real-time vs historical data.

**Decision:** Split into operational (hot, 7 days) and analytical (cold, months).

**Consequences:**

- Pros: Optimized performance for each use case, clear data lifecycle
- Cons: ETL complexity, data duplication, potential sync issues
- Alternative considered: Single database (simpler but slower queries)

---

### ADR-003: Use TimescaleDB Over ClickHouse

**Status:** Accepted

**Context:** Need time-series database for telemetry storage.

**Decision:** Use TimescaleDB (PostgreSQL extension) over ClickHouse.

**Consequences:**

- Pros: SQL familiarity, joins with metadata, simpler operations
- Cons: Lower max throughput than ClickHouse, more expensive at scale
- Alternative considered: ClickHouse (columnar, faster for analytics)

---

### ADR-004: Archive Raw Data to Parquet

**Status:** Accepted

**Context:** Need reprocessing capability and data science access.

**Decision:** Hourly batch raw messages to Parquet files in MinIO.

**Consequences:**

- Pros: Immutable source of truth, reprocessing, data science friendly
- Cons: Storage costs, ETL complexity, duplicate data
- Alternative considered: No archival (simpler but risky)

---

## Appendix B: Deployment Checklist

### Pre-Deployment

- [ ] VPS provisioned (8GB RAM minimum)
- [ ] Domain registered and DNS configured
- [ ] SSL certificates planned (Let's Encrypt via Traefik)
- [ ] Backup destination configured (external storage)
- [ ] Monitoring service setup (UptimeRobot)
- [ ] Alert channels configured (email/Slack)

### Initial Deployment

- [ ] Docker and Docker Compose installed
- [ ] Repository cloned
- [ ] .env file configured with strong secrets
- [ ] Firewall rules applied (SSH, HTTPS, MQTTS only)
- [ ] docker-compose up -d executed
- [ ] All health checks passing
- [ ] Grafana accessible and configured
- [ ] Test device successfully connects

### Post-Deployment

- [ ] Backup script configured and tested
- [ ] Monitoring dashboards reviewed
- [ ] Alert rules tested
- [ ] Documentation updated with deployment details
- [ ] Disaster recovery procedure tested
- [ ] Performance baseline established

---

## Appendix C: Resource Estimation

### Single VPS Requirements

**Minimum (1,000 devices):**

- 8GB RAM
- 4 vCPUs
- 100GB SSD
- 1TB bandwidth/month
- Cost: ~$40-60/month

**Recommended (5,000 devices):**

- 16GB RAM
- 8 vCPUs
- 250GB SSD
- 3TB bandwidth/month
- Cost: ~$80-120/month

**Maximum (10,000 devices):**

- 32GB RAM
- 16 vCPUs
- 500GB SSD
- 5TB bandwidth/month
- Cost: ~$200-300/month

### Storage Growth

**Per device per month:**

- Telemetry: ~2.5MB (1 msg/min, 60 bytes/msg)
- With compression: ~250KB (10:1 ratio)

**10,000 devices:**

- Monthly raw: 25GB
- Monthly compressed: 2.5GB
- Annual: 30GB compressed + aggregates

---

**End of Specification**

*This document is a living specification and will be updated as the project evolves.*
