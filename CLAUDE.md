# expense-lcl — Budgy App Development Log

## Project Overview
- **App:** Budgy (expense tracker) - Tauri 2.x mobile app
- **Frontend:** HTML/CSS/JS (no build step, served as-is from `src/`)
- **Backend:** Rust (Rust commands exposed to JS via `window.__invoke__`)
- **APK location:** `src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk`
- **Package:** `com.budgy.app`
- **Min SDK:** 26 (Android 8.0)

## Current Architecture
- LLM: Gemma 2B IT Q3_K_M GGUF (~1.5GB) — BUNDLED in APK assets; VPS fallback at `http://100.91.232.35:8088`
- Android app uses bundled GGUF (when llama-server Android binary available) or VPS LLM via Tailscale
- DB: SQLite via `init_db`, `add_expense`, `get_expenses`, etc.
- Receipt OCR: screenshot → base64 → LLM → structured JSON

## APK Size
- APK (without GGUF): ~184MB (includes 4 ABIs: arm64-v8a, armeabi-v7a, x86, x86_64)
- APK (with GGUF bundled): ~1.7GB universal debug APK
- GGUF model: 1.5GB — BUNDLED in APK assets ✅ (installed 2026-04-07)
- On-device llama-server: NOT yet compiled for Android (needs NDK cross-compile)
- App still connects to VPS LLM by default; on-device inference follows

## UI Fixes Applied (2026-04-06 / 2026-04-07)

### Chat button not responding
- Root cause: `toggleChat()` defined inside async IIFE in chat.js, never exposed to `window`
- Fix: Added `window.toggleChat = toggleChat;` at end of chat.js; removed inline `onclick` from HTML FAB
- Additional fix: Replaced keypress Enter handler with `<form onsubmit="sendChat(); return false;">` to avoid Android keyboard dismiss issues

### FAB positions (multiple revisions)
- Original: `bottom: 90px` (overlapped nav bar settings button)
- Revised to: `bottom: 160px` (CSS ID selector `#chat-fab` was overriding class `.chat-fab`)
- Revised to: `top: 80px; right: 24px` (user wanted lower right)
- Final: `bottom: 160px; right: 24px` (lower right above nav bar)
- Add button: Removed standalone FAB overlay, moved into bottom nav as 5th item "➕ Add"
- Nav order: Home | Transactions | Add | Categories | Settings (Add in middle position)

### Add button popup
- Tap ➕ in nav bar → shows modal with two options:
  - "📷 Open Camera" → `captureReceipt()`
  - "✏️ Add Manually" → shows quick-add form
- Modal CSS in `main.css` (.modal-overlay, .modal-content, .popup-btn)
- Event handlers in `app.js` DOMContentLoaded listener

## Rust Commands (window.__invoke__)
- `init_db` — initialize SQLite
- `add_expense` — insert expense
- `get_expenses` — query expenses
- `chat_query` — LLM chat (reqwest HTTP POST to VPS llama-server)
- `export_data` — export DB (stub, not implemented)
- `extract_receipt` — receipt OCR (not wired to Tauri command)

## Known Issues / TODO
- [ ] Chat send: Still testing if form submit approach works on Android
- [x] GGUF bundling: DONE ✅ (2026-04-07) — GGUF in APK assets, 1.7GB APK installed
- [ ] `export_data` command has no implementation in Rust
- [ ] `detect_recurring` command has no frontend caller
- [ ] `save_categories` backend expects `id` field, frontend doesn't send it
- [ ] Tauri notification permissions not configured
- [ ] Startup script for llama-server on VPS (auto-start on reboot)

## LLM Integration Details
- VPS: `http://100.91.232.35:8088` (Tailscale VPN)
- llama-server binary: `/tmp/llama-b8672/` (pre-built llama.cpp b8672)
- Model: `/home/clawbster/.openclaw/workspace/model-backups/expense-tracker/gemma-2b-it-Q3_K_M.gguf`
- llama-server PID on VPS: 439220 (check with `ps aux | grep llama`)
- llama-server port: 8088, bound to `0.0.0.0`
- App connects via HTTP POST `/v1/chat/completions`
- reqwest with rustls-tls + blocking feature (ureq was incompatible)

## File Locations
- Frontend source: `/home/clawbster/.openclaw/workspace/expense-lcl/src/`
- APK assets (staging): `/home/clawbster/.openclaw/workspace/expense-lcl/src-tauri/gen/android/app/src/main/assets/`
- Rust source: `/home/clawbster/.openclaw/workspace/expense-lcl/src-tauri/src/`
- Android SDK: `/home/clawbster/android-sdk/`
- NDK: `26.1.10909125`

## VPS Info
- Provider: OVH (Germany)
- Tailscale IP: 100.91.232.35
- Phone Tailscale: 100.81.58.89
- code-server: port 3006 (broken — requires Node v18, VPS has v22)
