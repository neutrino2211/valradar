# Introduction

<p align="center">
    <img src="https://raw.githubusercontent.com/neutrino2211/valradar/main/images/logo.png" alt="valradar logo" width="400"/>
</p>

**Valradar** is a high-performance, low-latency, and scalable data processing framework designed for OSINT, RECON, and a wide range of data collection operations.

## What is Valradar?

Valradar provides a flexible plugin architecture that allows you to create custom data collection and processing pipelines. Written in Rust for performance with a Python plugin system for flexibility, it combines the best of both worlds.

Think of it as a framework for building your own reconnaissance tools — whether you're extracting emails from websites, scanning for security vulnerabilities, or gathering intelligence from various sources.

## Why Valradar?

### 🚀 High Performance
Valradar leverages Rust's performance and Python's ecosystem. The core runtime handles multiprocessing and orchestration efficiently, while plugins benefit from Python's rich library ecosystem.

### 🔌 Plugin Architecture
Write plugins in Python using a clean, class-based API. The SDK provides type hints and IDE support for a great developer experience.

### 🔄 Recursive Processing
Built-in support for depth-based recursive data collection. Set how deep you want to crawl, and Valradar handles the rest.

### 📊 Parallel Processing
Configure concurrency to run multiple workers in parallel. Process large datasets quickly without writing threading code.

### 🛠️ Developer-Friendly CLI
A modern CLI with subcommands for running plugins, creating new ones, listing available plugins, and validating plugin structure.

## Key Features

| Feature | Description |
|---------|-------------|
| **Multiprocessing** | Parallel data processing with configurable worker count |
| **State Management** | Context class for maintaining state across processing steps |
| **Recursive Collection** | Depth-based recursive data gathering |
| **Plugin Discovery** | Automatic plugin discovery from configured paths |
| **Dependency Management** | Install plugin dependencies automatically |
| **Validation** | Built-in doctor command to validate plugin structure |

## Use Cases

Valradar excels at:

- **Email Harvesting** — Extract emails from websites and web pages
- **OSINT Operations** — Gather open-source intelligence from various sources
- **Security Reconnaissance** — Scan and enumerate targets
- **Web Scraping** — Collect structured data from websites
- **Data Processing Pipelines** — Transform and process data at scale

## Architecture Overview

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│    CLI      │ ──▶ │   Runtime   │ ──▶ │   Plugin    │
│  (Rust)     │     │   (Rust)    │     │  (Python)   │
└─────────────┘     └─────────────┘     └─────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │ Orchestrator│
                    │   Workers   │
                    └─────────────┘
```

The Rust runtime provides the CLI and orchestration layer, while plugins are written in Python. The runtime manages worker processes and coordinates data flow between collection and processing stages.

## Next Steps

Ready to get started? Head to the [Installation](./installation.md) guide to set up Valradar, or jump straight to the [Quick Start](./quick-start.md) to run your first plugin.
