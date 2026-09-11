# Helios: Universal Log Pre-processing Framework (ULPF)
**Architecture & Data Flow Document**

*This architecture is designed for extreme throughput, zero-downtime extensibility, and deployment in air-gapped or Big Data environments.*

## 1. High-Level Architecture Diagram

```mermaid
flowchart LR
    %% Styles
    classDef ingest fill:#e2e8f0,stroke:#64748b,stroke-width:2px,color:#0f172a
    classDef core fill:#dbeafe,stroke:#3b82f6,stroke-width:2px,color:#1e3a8a
    classDef dsl fill:#fef08a,stroke:#eab308,stroke-width:2px,color:#713f12
    classDef output fill:#dcfce7,stroke:#22c55e,stroke-width:2px,color:#14532d

    %% Ingestion Layer
    subgraph Ingestion["Ingestion Layer (Any Source)"]
        direction TB
        A[Perimeter Devices] --> B(File Upload / CLI)
        C[Syslog / Fluentd] --> D(REST API Ingest)
    end
    class B,D ingest

    %% Backend Engine
    subgraph Engine["Rust Pre-processing Engine (Helios Core)"]
        direction TB
        E{Format Detector}
        
        subgraph Parsers["Parsing Layer"]
            direction LR
            F[Native Rust Parsers<br>Syslog, CEF, EVTX]
            G[Dynamic Lisp Engine<br>Steel Scheme DSL]
        end
        
        H[Normalization Core<br>UniversalEvent Schema]
        
        E -->|Auto-routes| F
        E -->|Auto-routes| G
        F --> H
        G --> H
    end
    class E,H core
        class G dsl

    %% Output / UI
    subgraph Presentation["Delivery & Analytics Layer"]
        direction TB
        I[Axum REST API]
        J[Svelte 5 Dashboard<br>WebAssembly + ECharts]
        K[SIEM / Data Lake / AI]
    end
    class I,J,K output

    Ingestion --> E
    H --> I
    I -->|SSE Stream / JSON| J
    I -->|Standardized JSON| K
    
    %% AI Integration
    L((AI Co-Pilot)) -.-> J
```

## 2. Core Component Breakdown

**1. Multi-Modal Ingestion Layer**
Supports ingestion via static file uploads (Big Data batch processing), command-line execution, or real-time streaming via a high-throughput Axum REST API. Agnostic to the underlying hardware or perimeter device.

**2. Auto-Detection Engine (`helios-detector`)**
A zero-config routing layer that analyzes byte-signatures and regex heuristics of incoming logs to dynamically map them to the correct parser without user intervention.

**3. Hybrid Parsing Layer**
- **Native Parsers (`helios-parser`):** Hardcoded Rust implementations for standard, high-volume formats (Syslog, CEF, EVTX, JSON) ensuring maximum CPU efficiency.
- **Dynamic DSL Engine (`helios-lisp`):** An embedded Steel Scheme (Lisp) engine allowing security engineers to drop in custom parser scripts (`.scm`) at runtime. Satisfies the *"plug-and-play onboarding"* requirement with zero Rust recompilation.

**4. Normalization Core (`helios-core`)**
Transforms heterogeneous parsed data into a strict `UniversalEvent` schema. It enforces **Traceability** by appending the parsed fields alongside the pristine, untouched `raw_event` string to ensure zero information loss for legal/forensic compliance.

**5. Presentation & Delivery Layer (`web/`)**
A containerized, static Svelte 5 Single Page Application (SPA). Features infinite DOM virtualization and ECharts to render millions of events. Can be served via Caddy in an **air-gapped network**, while exposing structured JSON endpoints for seamless integration with external SIEMs and Data Lakes.
