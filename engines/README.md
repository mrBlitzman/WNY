# Bundled engines

This folder is empty in git. `scripts/fetch-engines.sh` fills `engines/cache/`.
`scripts/stage-engines.sh <triple>` copies the headless binaries into `engines/bin/`.
Release installers bundle that directory next to the WNY executable (`WNY_ENGINES_DIR`).

Headless only — never start the upstream GUI:

| id | binary names | slot |
| --- | --- | --- |
| dnscrypt-proxy | `dnscrypt-proxy` | Names |
| goodbyedpi | `goodbyedpi` | Filter (Windows) |
| zapret | `winws` / `tpws` | Filter |
| spoofdpi | `spoofdpi` | Filter (macOS) |
| byedpi | `ciadpi` | Filter |
| tor | `tor`, `lyrebird` | Road overlay + Snowflake/WebTunnel/obfs4 |
| hev-socks5-tunnel | `hev-socks5-tunnel` | TUN → Tor SOCKS |
| warp | `wgcf` + `wireguard-go` / `boringtun-cli` | Road |
| riseup | `bitmask-helper` | Road |
| psiphon | `psiphon-tunnel-core` | Road |
| lokinet | `lokinet` | Road overlay fallback |

Hashes: `policy/engines.lock`. Tamper → WNY refuses to start that engine.
