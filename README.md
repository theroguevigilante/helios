# Helios: The Swiss Army Knife of Log Preprocessing

**Helios** is a high-performance Universal Log Pre-processing & Observability Platform built natively in **Rust**. It automatically detects, parses, enriches, and normalizes log formats (from raw Syslog to binary EVTX) into a unified event schema, served through a modern **Svelte 5** frontend.

---

## Key Features

- **High-Performance Native Backend**: Built completely in Rust to tear through gigabytes of logs in seconds with a minimal memory footprint.
- **Infinite DOM Virtualization**: The frontend dashboard effortlessly renders millions of log events without lagging the browser.
- **Multi-Tab Forensic Workspace**: Instantly switch between the live log stream and static file analysis in isolated, state-preserving tabs.
- **Zero-Config Auto-Detect**: Drag and drop any log file to automatically detect its format and normalize it on the fly.
- **Interactive Visualization**: ECharts-powered timeline brushing to visually filter events.
- **Dynamic Lisp DSL Engine**: Write custom parsers on the fly using Steel Scheme (`.scm`) with hot-reloading support.
- **AI Forensic Co-pilot**: Bring Your Own Key (BYOK) AI chat interface powered by Gemini/OpenAI to analyze complex logs right inside the dashboard.

---

## Workspace Structure

- **`web/`**: Interactive web dashboard and landing page (SvelteKit, Svelte 5 Runes, Vite, Tailwind CSS v4, ECharts).
- **`helios-core`**: Core data models (such as `UniversalEvent`) and shared types.
- **`helios-parser`**: Parser traits, registry, and metadata definitions.
- **`helios-detector`**: Log format detection engine.
- **`helios-ingest`**: Log ingestion pipelines and sources (e.g., File, Evtx, Stdout).
- **`helios-api`**: Axum-based REST API with endpoints for uploading, stats, and SSE streaming.
- **`helios-cli`**: Command-line interface for running format detection, parsing, normalization, and starting the API server.
- **`parsers/`**: Modular parser plugins for Syslog, JSON, Apache, Nginx, CEF, ZooKeeper, Spark, Windows (EVTX), Android Logcat, OpenSSH, and Proxifier.

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

Ensure [Node.js](https://nodejs.org/) (v20+) is installed.

---

## Deploying for Production

Helios compiles to standalone binaries and static assets, giving you complete flexibility in how you deploy it.

### Option A: Docker / Podman (Recommended)
We provide multi-stage Dockerfiles that build the Rust backend and the static Svelte frontend, serving everything cleanly via a minimal containerized Caddy instance.

```bash
# Using Docker Compose (or Podman Compose)
docker-compose up -d --build
```
This deploys:
- `frontend`: Serves the static Svelte SPA on port `80` and reverse-proxies `/api` traffic.
- `backend`: Runs the Rust engine on port `8080` (accessible internally).

### Option B: Bare Metal with Caddy (Live at [helios.naimish.xyz](https://helios.naimish.xyz))
You can run the `helios-cli` binary directly on a server and use a Reverse Proxy like Caddy to serve the static frontend and route API requests.

**1. Build the Static SPA:**
```bash
cd web
npm i -D @sveltejs/adapter-static
npm run build
```
Rsync the `web/build/` directory to `/var/www/helios` on your server.

**2. Example Caddyfile (`/etc/caddy/Caddyfile`):**
```caddyfile
helios.naimish.xyz {
    # API Routing
    handle /api/* {
        # Secure the live-feed to internal networks only (optional)
        @liveFeed {
            path /api/v1/stream* /api/v1/ingest*
        }
        respond @liveFeed "Live feed requires internal VPN access." 403

        # Proxy standard API requests to the Rust backend
        reverse_proxy 127.0.0.1:8080
    }

    # Serve the static Svelte frontend
    handle {
        root * /var/www/helios
        try_files {path} /index.html
        file_server
    }
}
```

### Option C: Cross-Compiling for ARM64
When deploying to an ARM64 server with older GLIBC versions (e.g., Ubuntu 22.04), you can bypass version errors by using `cargo-zigbuild` to compile a highly compatible binary:

```bash
cargo install cargo-zigbuild
cargo zigbuild --release --target aarch64-unknown-linux-gnu.2.39
```

---

## Local Development Stack

### 1. Start the Rust Backend API Server
In the root directory, start the API server on port `8080`:

```bash
cargo run -p helios-cli -- serve --port 8080
```

### 2. Start the Frontend Development Server
In a new terminal tab/window:

```bash
cd web
npm install
npm run dev
```

Open [http://localhost:5173](http://localhost:5173) in your browser. The frontend automatically proxies `/api` requests to the Rust backend.

---

## CLI Usage

### 1. Parse a Log File
Run the parser on a log file. The detector will identify the format and output structured JSON events:

```bash
cargo run -p helios-cli -- parse --file <path-to-log-file>
```

### 2. Format Detection Only
Detect the format of a log file without parsing the full event payload:

```bash
cargo run -p helios-cli -- detect --file <path-to-log-file>
```

---

## REST API Endpoints

When running `cargo run -p helios-cli -- serve --port 8080`:

| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/api/v1/stats` | System statistics & active parsers |
| `GET` | `/api/v1/stream` | Server-Sent Events (SSE) live feed of normalized logs |
| `POST` | `/api/v1/events` | Ingest live raw log events |
| `POST` | `/api/v1/upload` | Upload a static file (e.g., `.evtx`, `.log`) for parsing and JSON extraction |

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

---

## Citation

The dataset is from loghub: 
+ **Loghub**: Jieming Zhu, Shilin He, Pinjia He, Jinyang Liu, Michael R. Lyu. [Loghub: A Large Collection of System Log Datasets for AI-driven Log Analytics](https://arxiv.org/abs/2008.06448). IEEE International Symposium on Software Reliability Engineering (ISSRE), 2023.
+ **Loghub-2.0**: Zhihan Jiang, Jinyang Liu, Junjie Huang, Yichen Li, Yintong Huo, Jiazhen Gu, Zhuangbin Chen, Jieming Zhu, Michael R. Lyu. [A Large-scale Evaluation for Log Parsing Techniques: How Far are We?](https://arxiv.org/abs/2308.10828). ACM SIGSOFT International Symposium on Software Testing and Analysis (ISSTA), 2024.
