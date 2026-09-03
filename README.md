# Helios: The Swiss Army Knife of Log Preprocessing

**Helios** is a high-performance Universal Log Pre-processing & Observability Platform built natively in **Rust** with a modern **Svelte 5** frontend. It automatically detects, parses, enriches, and normalizes unstructured and structured log formats—from raw Syslog to complex EVTX binaries—into a unified event schema.

---

## 🚀 Key Features

- **Blazing Fast Native Backend**: Built completely in Rust. Zero JVM bloat. Handles gigabytes of unparsed logs in seconds.
- **Infinite DOM Virtualization**: The frontend dashboard seamlessly renders millions of log events without crashing or lagging the browser.
- **Multi-Tab Forensic Workspace**: Instantly switch between the live log stream and static file analysis in isolated, state-preserving tabs.
- **Zero Config Auto-Detect**: Drag and drop any log file. The AI-ready registry detects and normalizes it on the fly.
- **Interactive Visualization**: ECharts-powered timeline brushing to filter events interactively.

---

## 📂 Workspace Structure

- **`web/`**: Interactive web dashboard and landing page (SvelteKit, Svelte 5 Runes, Vite, Tailwind CSS v4, ECharts).
- **`helios-core`**: Core data models (such as `UniversalEvent`) and shared types.
- **`helios-parser`**: Parser traits, registry, and metadata definitions.
- **`helios-detector`**: Automatic log format detection engine.
- **`helios-ingest`**: Log ingestion pipelines and sources (e.g., File, Evtx, Stdout).
- **`helios-api`**: Axum-based high-performance REST API with endpoints for uploading, stats, and SSE streaming.
- **`helios-cli`**: Command-line interface for running format detection, parsing, normalization, and starting the API server.
- **`parsers/`**: Modular parser plugins for Syslog, JSON, Apache, Nginx, CEF, ZooKeeper, Spark, Windows (EVTX), Android Logcat, OpenSSH, and Proxifier.

---

## 🛠️ Prerequisites & Installation

### 1. Install Rust & Cargo

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After installation, reload your shell environment so `cargo` is available in your `$PATH`:

```bash
source "$HOME/.cargo/env"
```

### 2. Install Node.js & npm (for Frontend)

Make sure [Node.js](https://nodejs.org/) (v20+) is installed.

---

## ⚡ Running the Application

### Option A: Run Full Stack (Backend API + Frontend UI)

#### 1. Start the Rust Backend API Server
In the root directory, start the API server on port `8080`:

```bash
cargo run -p helios-cli -- serve --port 8080
```

#### 2. Start the Frontend Development Server
In a new terminal tab/window:

```bash
cd web
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

#### 2. Format Detection Only
Detect the format of a log file without parsing the full event payload:

```bash
cargo run -p helios-cli -- detect --file <path-to-log-file>
```

---

## 🌐 REST API Endpoints

When running `cargo run -p helios-cli -- serve --port 8080`:

| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/api/v1/stats` | System statistics & active parsers |
| `GET` | `/api/v1/stream` | Server-Sent Events (SSE) live feed of normalized logs |
| `POST` | `/api/v1/events` | Ingest live raw log events |
| `POST` | `/api/v1/upload` | Upload a static file (e.g., `.evtx`, `.log`) for instant parsing and JSON extraction |

---

## 🧪 Development & Testing

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

---

## 📚 Citation

The dataset is from loghub: 
+ **Loghub**: Jieming Zhu, Shilin He, Pinjia He, Jinyang Liu, Michael R. Lyu. [Loghub: A Large Collection of System Log Datasets for AI-driven Log Analytics](https://arxiv.org/abs/2008.06448). IEEE International Symposium on Software Reliability Engineering (ISSRE), 2023.
+ **Loghub-2.0**: Zhihan Jiang, Jinyang Liu, Junjie Huang, Yichen Li, Yintong Huo, Jiazhen Gu, Zhuangbin Chen, Jieming Zhu, Michael R. Lyu. [A Large-scale Evaluation for Log Parsing Techniques: How Far are We?](https://arxiv.org/abs/2308.10828). ACM SIGSOFT International Symposium on Software Testing and Analysis (ISSTA), 2024.

---

