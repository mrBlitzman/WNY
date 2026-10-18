# Bundled engines

This folder is empty in git. `scripts/fetch-engines.sh` fills `engines/cache/`.
Release builds copy the verified binaries into `engines/bin/` beside the WNY executable.

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
