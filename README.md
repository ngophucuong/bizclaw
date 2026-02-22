# ⚡ BizClaw

> **Hạ tầng AI Agent nhanh, module hoá — viết hoàn toàn bằng Rust.**

BizClaw là nền tảng AI Agent kiến trúc trait-driven, có thể chạy **mọi nơi** — từ Raspberry Pi đến cloud server. Hỗ trợ nhiều LLM provider, kênh giao tiếp, và công cụ thông qua kiến trúc thống nhất, hoán đổi được.

[![Rust](https://img.shields.io/badge/Rust-100%25-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-66%20passing-brightgreen)]()
[![Crates](https://img.shields.io/badge/crates-12%2F12-success)]()

---

## 🇻🇳 Tiếng Việt

### 🚀 100% Tự Host — Không phụ thuộc Cloud

- **100% Độc lập:** Clone về là chạy — laptop, VPS, hay Raspberry Pi. Không token khoá, không telemetry.
- **Dữ liệu nội bộ:** Chat history, API Keys mã hoá AES-256 lưu local.
- **Offline AI:** Brain Engine chạy LLM offline (Llama, DeepSeek) — tối ưu cho 512MB RAM.

### 🎯 Tính năng

| Hạng mục | Chi tiết |
|----------|----------|
| **🧠 Brain Engine** | LLaMA inference: GGUF, mmap, quantization, Flash Attention, FP16 KV Cache |
| **🔌 8 Providers** | OpenAI, Anthropic, Ollama, llama.cpp, Brain, Gemini, DeepSeek, Groq |
| **💬 6 Channels** | CLI, Zalo Personal, Telegram, Discord (Gateway WS), Email (IMAP/SMTP), Webhook |
| **🏢 Multi-Tenant** | Admin Platform, JWT Auth, Tenant Manager, Pairing Codes, Audit Log |
| **🌐 Web Dashboard** | Chat UI (VI/EN), WebSocket real-time, embedded SPA |
| **🛠️ 5 Tools** | Shell, File, Web Search, Group Summarizer, Google Calendar |
| **🔒 Security** | Command allowlist, AES-256, HMAC-SHA256, JWT + bcrypt |
| **💾 Memory** | SQLite + RAG-style retrieval, keyword search, relevance scoring |
| **⚡ SIMD** | ARM NEON, x86 SSE2/AVX2 auto-dispatch |

### 🏗️ Kiến trúc

```
┌─────────────────────────────────────────────────┐
│                 bizclaw (CLI)                     │
│          ┌──────────────────┐                     │
│          │  bizclaw-agent   │ ← RAG Memory        │
│          │  Multi-round     │   + Tool Calling     │
│          │  Tool Calling    │   (max 3 rounds)     │
│          └──────┬───────────┘                     │
│    ┌────────────┼─────────────┐                   │
│    ▼            ▼             ▼                   │
│ Providers    Channels       Tools                 │
│ ────────    ────────       ─────                 │
│ OpenAI      CLI            Shell                 │
│ Anthropic   Zalo           File                  │
│ Ollama      Telegram       Web Search            │
│ Gemini      Discord        Calendar              │
│ DeepSeek    Email          Group Summarizer       │
│ Groq        Webhook                              │
│ Brain                                            │
│    ┌────────────┼─────────────┐                   │
│    ▼            ▼             ▼                   │
│ Memory       Security      Gateway               │
│ ────────    ────────       ─────                 │
│ SQLite      Allowlist      Axum HTTP             │
│ RAG         AES-256        WebSocket             │
│ Vector      Sandbox        REST API              │
│                                                   │
│          ┌──────────────────┐                     │
│          │  bizclaw-brain   │                     │
│          │  GGUF + SIMD     │                     │
│          │  Offline LLM     │                     │
│          └──────────────────┘                     │
└─────────────────────────────────────────────────┘
```

### 🚀 Bắt đầu nhanh

```bash
# Clone và build
git clone https://github.com/nguyenduchoai/bizclaw.git
cd bizclaw
cargo build --release

# Cài đặt (wizard tương tác)
./target/release/bizclaw init

# Chat ngay (interactive CLI)
./target/release/bizclaw agent --interactive

# Chat 1 câu
./target/release/bizclaw agent -m "Xin chào!"

# Mở Web Dashboard (single tenant)
./target/release/bizclaw serve
```

### 🏢 Chế độ triển khai

BizClaw hỗ trợ **2 chế độ chạy**:

#### 1. Standalone Mode — Một tenant duy nhất

Phù hợp cho: cá nhân, startup nhỏ, test/demo.

```bash
# Chỉ cần binary `bizclaw` — KHÔNG cần bizclaw-platform
./target/release/bizclaw serve --port 3000

# Hoặc chạy channels trực tiếp
./target/release/bizclaw channel start --all
```

- Không cần Admin Platform
- Config bằng file `~/.bizclaw/config.toml`
- Web Dashboard tại `localhost:3000`
- Quản lý channels qua CLI hoặc dashboard

#### 2. Platform Mode — Multi-Tenant

Phù hợp cho: agency, nhiều bots, production server.

```bash
# Cần build cả 2 binaries
cargo build --release --bin bizclaw --bin bizclaw-platform

# Khởi tạo admin user
./target/release/bizclaw-platform --init-admin

# Chạy platform (quản lý nhiều tenants)
./target/release/bizclaw-platform --port 3001

# Mỗi tenant sẽ được tạo qua Admin Dashboard
# và tự động chạy trên port riêng (10001, 10002, ...)
```

- Admin Dashboard tại `http://localhost:3001`
- Mỗi tenant là 1 process `bizclaw serve` riêng
- Tenant quản lý qua REST API hoặc Web UI
- JWT Auth + Pairing Code cho bảo mật

### 🧠 Ollama / Brain Engine — Shared Models

Ollama models được **dùng chung** giữa tất cả tenants:

```
┌─────────────────────────────────────────┐
│         Ollama Server (shared)           │
│         localhost:11434                   │
│         ┌─────────────────┐             │
│         │ tinyllama (1.5GB)│             │
│         │ llama3.2  (3.8GB)│             │
│         └─────────────────┘             │
│              ▲    ▲    ▲                │
│              │    │    │                │
│  Tenant A ───┘    │    └─── Tenant C    │
│  (ollama/         │         (openai/    │
│   tinyllama)      │          gpt-4o)    │
│              Tenant B                    │
│              (ollama/                    │
│               llama3.2)                  │
└─────────────────────────────────────────┘
```

- **Pull model 1 lần** → tất cả tenant dùng được
- **RAM:** ~2-4GB cho 7B model (chỉ 1 model active cùng lúc)
- **Mỗi tenant chọn model riêng** trong config (provider + model)
- **Cloud fallback:** Nếu không đủ RAM → dùng OpenAI, Anthropic, Gemini

```bash
# Cài Ollama trên server
curl -fsSL https://ollama.ai/install.sh | sh

# Pull model nhẹ (~1.5GB)
ollama pull tinyllama

# Hoặc model mạnh hơn (~3.8GB, cần 4GB+ RAM)
ollama pull llama3.2
```

### ⚙️ Cấu hình

File config tại `~/.bizclaw/config.toml`:

```toml
default_provider = "ollama"    # hoặc "openai", "anthropic", "gemini"
default_model = "tinyllama"
default_temperature = 0.7

[identity]
name = "BizClaw"
persona = "Trợ lý AI thông minh"
system_prompt = "Bạn là BizClaw, trợ lý AI nhanh và có năng lực."

[brain]
enabled = false                # true = dùng Brain Engine (offline)
model_path = "~/.bizclaw/models/tinyllama.gguf"

[memory]
backend = "sqlite"
auto_save = true

[gateway]
enabled = true
host = "127.0.0.1"
port = 3000

[autonomy]
level = "supervised"
allowed_commands = ["ls", "cat", "echo", "pwd", "find", "grep"]
```

### 📦 Crate Map

| Crate | Mô tả | Trạng thái |
|-------|--------|------------|
| `bizclaw-core` | Traits, types, config, errors | ✅ |
| `bizclaw-brain` | GGUF inference + SIMD | ✅ |
| `bizclaw-providers` | 8 LLM providers | ✅ |
| `bizclaw-channels` | 6 channels (CLI, Zalo, TG, Discord, Email, Webhook) | ✅ |
| `bizclaw-memory` | SQLite + RAG retrieval | ✅ |
| `bizclaw-tools` | 5 tools (Shell, File, Search, Calendar, Summarizer) | ✅ |
| `bizclaw-security` | Allowlist, AES-256, Sandbox | ✅ |
| `bizclaw-agent` | Agent loop + multi-round tool calling | ✅ |
| `bizclaw-gateway` | Axum HTTP + WebSocket + Dashboard | ✅ |
| `bizclaw-runtime` | Native process adapter | ✅ |
| `bizclaw-platform` | Multi-tenant admin platform | ✅ |

### 🔒 Bảo mật

| Tính năng | Mô tả |
|-----------|--------|
| **Allowlist** | Chỉ lệnh được phép mới thực thi |
| **Path Restrictions** | Chặn `~/.ssh`, `/etc` |
| **Sandbox** | Timeout, cắt output |
| **AES-256** | Mã hoá key (hostname+user) |
| **JWT + bcrypt** | Admin Platform auth |
| **HMAC-SHA256** | Webhook signature |

### 📡 Gateway API

| Endpoint | Method | Mô tả |
|----------|--------|--------|
| `/health` | GET | Health check |
| `/api/v1/info` | GET | System info |
| `/api/v1/config` | GET | Config (sanitized) |
| `/api/v1/providers` | GET | Available providers |
| `/api/v1/channels` | GET | Channel list |
| `/ws` | WS | Real-time chat |

### 📁 Project Structure

```
bizclaw/
├── Cargo.toml                 # Workspace root
├── src/
│   ├── main.rs                # bizclaw CLI binary
│   └── platform_main.rs       # bizclaw-platform binary
├── crates/
│   ├── bizclaw-core/          # Traits, types, config
│   ├── bizclaw-brain/         # Local GGUF inference
│   ├── bizclaw-providers/     # LLM providers (8)
│   ├── bizclaw-channels/      # Communication (6 channels)
│   ├── bizclaw-memory/        # SQLite + RAG
│   ├── bizclaw-tools/         # Tools (5)
│   ├── bizclaw-security/      # AES-256, Sandbox
│   ├── bizclaw-agent/         # Agent engine
│   ├── bizclaw-gateway/       # HTTP + WebSocket + Dashboard
│   ├── bizclaw-runtime/       # Process adapters
│   └── bizclaw-platform/      # Multi-tenant admin
└── deploy/                    # Deployment configs
```

### 🧪 Testing

```bash
# Chạy tất cả tests
cargo test --workspace

# Test từng crate
cargo test -p bizclaw-brain     # Brain engine (12 tests)
cargo test -p bizclaw-core      # Core types (11 tests)
cargo test -p bizclaw-tools     # Tools (5 tests)
cargo test -p bizclaw-agent     # Agent (4 tests)
cargo test -p bizclaw-gateway   # Gateway (4 tests)
```

### 🚀 Production Deployment

```bash
# 1. Build release binaries
cargo build --release

# 2a. Standalone (1 bot)
cp target/release/bizclaw /usr/local/bin/
bizclaw init
bizclaw serve --port 3000

# 2b. Platform (nhiều bots)
cp target/release/bizclaw target/release/bizclaw-platform /usr/local/bin/
bizclaw-platform --init-admin --port 3001

# 3. Systemd service
sudo tee /etc/systemd/system/bizclaw-platform.service << 'EOF'
[Unit]
Description=BizClaw Multi-Tenant Platform
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/bizclaw-platform --port 3001
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
sudo systemctl enable --now bizclaw-platform

# 4. Nginx reverse proxy (optional)
# admin.yourdomain.com → :3001
# bot1.yourdomain.com  → :10001
```

### 📊 Stats

| Metric | Value |
|--------|-------|
| **Language** | 100% Rust |
| **Crates** | 12 (11 library + 1 binary) |
| **Lines of Code** | ~11,200 |
| **Tests** | 66 passing |
| **Providers** | 8 |
| **Channels** | 6 |
| **Tools** | 5 |
| **Binary Size** | bizclaw 7.6MB, bizclaw-platform 5.6MB |
| **RAM (idle)** | ~1.8MB |

---

## 🇬🇧 English

### Features

- **🧠 Brain Engine** — Local LLaMA inference via GGUF with SIMD
- **🔌 8 Providers** — OpenAI, Anthropic, Ollama, llama.cpp, Brain, Gemini, DeepSeek, Groq
- **💬 6 Channels** — CLI, Zalo, Telegram, Discord, Email (IMAP/SMTP), Webhook
- **🏢 Multi-Tenant Platform** — Admin dashboard, JWT auth, tenant lifecycle
- **🌐 Web Dashboard** — Bilingual (VI/EN), real-time WebSocket chat
- **🛠️ 5 Tools** — Shell, File, Web Search, Group Summarizer, Calendar
- **🔒 Security** — AES-256, Command allowlists, sandbox, HMAC-SHA256
- **💾 RAG Memory** — SQLite with keyword search and relevance scoring
- **⚡ SIMD** — ARM NEON, x86 SSE2/AVX2 auto-dispatch

### Quick Start

```bash
git clone https://github.com/nguyenduchoai/bizclaw.git
cd bizclaw && cargo build --release

# Standalone (single bot)
./target/release/bizclaw init
./target/release/bizclaw agent --interactive

# Platform (multi-tenant)
./target/release/bizclaw-platform --init-admin
./target/release/bizclaw-platform --port 3001
```

### Deployment Modes

| Mode | Binary | Use Case |
|------|--------|----------|
| **Standalone** | `bizclaw` only | Single bot, personal use, testing |
| **Platform** | `bizclaw` + `bizclaw-platform` | Multiple bots, agency, production |

### Ollama Shared Models

All tenants share the same Ollama instance. Pull a model once, every tenant can use it.

```bash
curl -fsSL https://ollama.ai/install.sh | sh
ollama pull tinyllama    # ~1.5GB, good for 2GB RAM
ollama pull llama3.2     # ~3.8GB, needs 4GB+ RAM
```

Each tenant selects its own provider/model in config. Cloud providers (OpenAI, etc.) work without Ollama.

---

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

---

**BizClaw** — *AI nhanh, mọi nơi. / Fast AI, everywhere.*
