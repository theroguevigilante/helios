# Helios

**Helios** is a high-performance Universal Log Pre-processing & Observability Platform built in Rust with a modern React/TypeScript frontend. It automatically detects, parses, enriches, and normalizes unstructured and structured log formats into a unified event schema.

---

## Workspace Structure

The project is organized into modular Rust crates and a modern React frontend:

- **`frontend/`**: Interactive web dashboard and real-time log processing playground (React 19, Vite, Tailwind CSS, shadcn/ui).
- **`helios-core`**: Core data models (such as `UniversalEvent`) and shared types.
- **`helios-parser`**: Parser traits, registry, and metadata definitions.
- **`helios-detector`**: Automatic log format detection engine.
- **`helios-enrichment`**: Context enrichment pipeline (e.g., GeoIP, threat intel, user metadata).
- **`helios-ingest`**: Log ingestion pipelines and sources.
- **`helios-storage`**: Storage layer and persistence backends.
- **`helios-api`**: Axum-based high-performance REST API with CORS and endpoints for detection, parsing, normalization, and stats.
- **`helios-cli`**: Command-line interface for running format detection, parsing, normalization, and starting the API server.
- **`parsers/`**: Modular parser plugins:
  - `helios-parser-syslog`: RFC 3164 (BSD syslog) and RFC 5424 syslog parser.
  - `helios-parser-json`: Structured JSON log parser.
  - `helios-parser-apache`: Apache Common/Combined access log parser.
  - `helios-parser-nginx`: Nginx access log parser.
  - `helios-parser-cef`: Common Event Format (CEF) log parser.
  - `helios-parser-zookeeper`: Apache ZooKeeper log parser.
  - `helios-parser-spark`: Apache Spark log parser.
  - `helios-parser-windows`: Windows CBS / application log parser.
  - `helios-parser-android`: Android logcat (threadtime format) parser.
  - `helios-parser-openssh`: OpenSSH authentication and server log parser.
  - `helios-parser-proxifier`: Proxifier client proxy log parser.


---

## Prerequisites & Installation

### 1. Install Rust & Cargo

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After installation, reload your shell environment so `cargo` is available in your `$PATH`:

```bash
source "$HOME/.cargo/env"
```

### 2. Install Node.js & npm (for Frontend)

Make sure [Node.js](https://nodejs.org/) (v18+) is installed.

---

## Running the Application

### Option A: Run Full Stack (Backend API + Frontend UI)

#### 1. Start the Rust Backend API Server
In the root directory, start the API server on port `8080`:

```bash
cargo run -p helios-cli -- serve --port 8080
```

#### 2. Start the Frontend Development Server
In a new terminal tab/window:

```bash
cd frontend
npm install
npm run dev
```

Open [http://localhost:5173](http://localhost:5173) in your browser. The frontend will automatically proxy `/api` requests to the Rust backend at `http://localhost:8080`.

---

### Option B: CLI Usage

#### 1. Parse a Log File
Run the parser on any log file using the CLI. The detector will automatically identify the log format and output structured JSON events:

```bash
cargo run -p helios-cli -- parse --file <path-to-log-file>
```

*Or with the short flag `-f`:*

```bash
cargo run -p helios-cli -- parse -f <path-to-log-file>
```

*When testing, you can redirect the output to a text file:*

```bash
cargo run parse --file <input-log-file> > <output-file.txt>
```

#### 2. Format Detection Only
Detect the format of a log file without parsing the full event payload:

```bash
cargo run -p helios-cli -- detect --file <path-to-log-file>
```

#### 3. Normalize Logs
Normalize logs into the standard `UniversalEvent` schema:

```bash
cargo run -p helios-cli -- normalize --file <path-to-log-file>
```

---

## REST API Endpoints

When running `cargo run -p helios-cli -- serve --port 8080`:

| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/api/v1/health` | Health check & service version |
| `GET` | `/api/v1/parsers` | List registered parser plugins & metadata |
| `POST` | `/api/v1/detect` | Detect log format (`{ "log": "<raw_log>" }`) |
| `POST` | `/api/v1/parse` | Parse raw log into `UniversalEvent` |
| `POST` | `/api/v1/normalize` | Detect, parse, and enrich into normalized event |
| `GET` | `/api/v1/stats` | System statistics & active parsers |

---

## Development & Testing

### Build the Rust Workspace

```bash
cargo build
```

### Run Rust Tests

```bash
cargo test
```

### Linting & Formatting

```bash
cargo clippy -- -D warnings
cargo fmt -- --check
```

### Build Frontend Bundle

```bash
cd frontend
npm run build
```

---

## Citation

The dataset is from loghub: 
+ **Loghub**: Jieming Zhu, Shilin He, Pinjia He, Jinyang Liu, Michael R. Lyu. [Loghub: A Large Collection of System Log Datasets for AI-driven Log Analytics](https://arxiv.org/abs/2008.06448). IEEE International Symposium on Software Reliability Engineering (ISSRE), 2023.
+ **Loghub-2.0**: Zhihan Jiang, Jinyang Liu, Junjie Huang, Yichen Li, Yintong Huo, Jiazhen Gu, Zhuangbin Chen, Jieming Zhu, Michael R. Lyu. [A Large-scale Evaluation for Log Parsing Techniques: How Far are We?](https://arxiv.org/abs/2308.10828). ACM SIGSOFT International Symposium on Software Testing and Analysis (ISSTA), 2024.


---

## License

MIT / Apache-2.0

