# WNY desktop (Windows + macOS)

Same Tauri shell on both. Filter engines differ:

- Windows: GoodbyeDPI → zapret `winws` → ByeDPI
- macOS: SpoofDPI → zapret `tpws` → ByeDPI

Neither launches an upstream window. Release builds bundle `engines/bin/` via Tauri resources.
For a local build: `./scripts/stage-engines.sh <triple>` then `npm run tauri build`.

Requires **Rust 1.90+** (Tauri 2). This crate is excluded from the workspace so `cargo test` still runs on 1.83 CI.

```bash
# from repo root
export WNY_DEV=1   # allow empty hashes while developing
cd apps/desktop
npm install
npm run tauri build
```

Linux is supported for development only (`cargo run -p wny-cli`).
