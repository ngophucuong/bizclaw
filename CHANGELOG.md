# Changelog — BizClaw

## [2026-02-27] — Deploy v2 + 4 New Providers + PageIndex RAG

### Added
- **ByteDance ModelArk** provider — Seed 1.6, Doubao 1.5 Pro 256K/32K (`ARK_API_KEY`)
- **Mistral** provider — mistral-large, mistral-small (`MISTRAL_API_KEY`)
- **MiniMax** provider — MiniMax-Text-01 1M context (`MINIMAX_API_KEY`)
- **xAI (Grok)** provider — grok-3, grok-3-mini (`XAI_API_KEY`)
- **PageIndex MCP** integration — vectorless reasoning-based RAG (98.7% FinanceBench)
- Cross-compilation support via `cargo-zigbuild` (Mac → Linux x86_64)
- `deploy.sh` — automated VPS deployment script (SCP + SSH)
- OpenSSL vendored feature for Linux cross-compilation
- Provider aliases: `grok`→xai, `bytedance`/`doubao`/`ark`/`volcengine`→modelark

### Changed
- Total AI providers: 11 → **15** built-in
- Knowledge RAG description: FTS5/BM25 + PageIndex MCP (dual-mode)
- README: PageIndex as first MCP example

### Removed
- GoClaw/OpenFang/OpenClaw references from README
- Inspiration & Credits section

### Infrastructure
- VPS: `116.118.2.98` running binary v2 (commit `ee70345`)
- Processes: bizclaw-platform (port 3001) + 3 tenant gateways (10001, 10002, 10004)
- SSL: Active for apps.bizclaw.vn, bizclaw.vn, apps.viagent.vn, viagent.vn

---

## [2026-02-27] — Dashboard Complete (18/18 Pages)

### Added
- ChatPage with WebSocket streaming
- All dashboard pages completed (18/18)
- Orchestration UI with delegate form

### Fixed
- Clippy cleanup: 122→8 warnings
- No-cache headers for dashboard JS
- Auth fix for orchestration
