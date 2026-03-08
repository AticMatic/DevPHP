# 🚀 DevPHP

**The modern, lightning-fast PHP development environment for Windows & macOS.**

Built with **Rust** and **Tauri**, DevPHP provides a zero-config, portable environment for PHP, MySQL, and Nginx without the bloat of traditional installers or the overhead of Docker.

## ⚡ Why DevPHP?

Most local PHP environments are either too heavy (Docker), too dated (XAMPP), or platform-locked. DevPHP is designed to be:

* **Native & Fast:** No virtualization. We manage portable binaries directly on your OS.
* **Isolated:** Everything lives in `~/.devphp`. No global environment variables or system-wide services.
* **Modern Stack:** A high-performance **Rust** engine with a beautiful **React** frontend.
* **One-Click SSL:** Automatic local trust store management for `https://`.

---

## 🏗 Architecture

DevPHP is strictly split into three layers to ensure scalability and ease of contribution:

1. **Desktop UI (React/Tauri):** A lightweight interface for service control and site management.
2. **Core Engine (Rust):** High-performance logic for process management, binary extraction, and filesystem operations.
3. **Portable Runtimes:** Pre-configured binaries of PHP, MySQL, and Nginx managed by the Core Engine.

---

## 🛠 Project Structure

```text
devphp/
├── apps/desktop/          # Tauri + React Frontend
├── crates/core/           # The "Brain" (Rust Engine)
│   ├── system/            # Process & Permission management
│   ├── binaries/          # Runtime downloader & extractor
│   ├── services/          # PHP, MySQL, Nginx controllers
│   └── sites/             # VHost & Hosts file automation
└── docs/                  # Architecture & Contribution guides

```

---

## 🚦 Roadmap (MVP)

* [ ] **Phase 1:** Core Rust `process_manager` (Start/Stop PHP & MySQL).
* [ ] **Phase 2:** Binary Registry (Auto-downloading portable runtimes).
* [ ] **Phase 3:** Automated Virtual Hosts (`project.test` mapping).
* [ ] **Phase 4:** System Tray integration & Live Logs.

---

## 🤝 Contributing

We love contributors! Whether you are a Rustacean, a TypeScript wizard, or a PHP enthusiast, there’s a place for you.

1. **Check the Issues:** Look for `good-first-issue` labels.
2. **Architecture First:** Please read our `docs/architecture.md` before submitting a PR.
3. **Vibe Check:** We aim for clean, documented, and type-safe code.

---

## 💻 Tech Stack

* **Language:** Rust 🦀
* **Frontend:** React + Tailwind CSS
* **Bridge:** Tauri
* **Runtimes:** PHP (8.3+), Nginx, MariaDB

---
