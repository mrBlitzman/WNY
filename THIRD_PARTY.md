# Third-party software

WNY orchestrates existing tools. Distributed builds that bundle engines inherit those licenses. This tree’s own code is GPL-3.0-or-later so a combined Windows/macOS/Android/iOS build can include GPL helpers.

| Component | Upstream | License (typical) | Role in WNY |
| --- | --- | --- | --- |
| dnscrypt-proxy | [DNSCrypt/dnscrypt-proxy](https://github.com/DNSCrypt/dnscrypt-proxy) | ISC | Names |
| GoodbyeDPI | [ValdikSS/GoodbyeDPI](https://github.com/ValdikSS/GoodbyeDPI) | Apache-2.0 + WinDivert | Filter (Windows) |
| WinDivert | [basil00/WinDivert](https://github.com/basil00/WinDivert) | LGPLv3 | Filter driver (Windows) |
| zapret | [bol-van/zapret](https://github.com/bol-van/zapret) | MIT | Filter |
| SpoofDPI | [xvzc/spoofdpi](https://github.com/xvzc/spoofdpi) | Apache-2.0 | Filter (macOS) |
| ByeDPI | [hufrea/byedpi](https://github.com/hufrea/byedpi) | MIT | Filter |
| Tor Expert Bundle / lyrebird | [The Tor Project](https://www.torproject.org/download/tor/) | BSD-3-Clause | Overlay |
| Arti / onionmasq | [tpo/core/onionmasq](https://gitlab.torproject.org/tpo/core/onionmasq) | Apache-2.0 / MIT | Overlay (Android) |
| hev-socks5-tunnel | [heiher/hev-socks5-tunnel](https://github.com/heiher/hev-socks5-tunnel) | MIT | TUN → SOCKS |
| WireGuard / boringtun | WireGuard / Cloudflare | GPL-2.0 / BSD | WARP transport |
| bitmask-vpn helpers | [leapcode/bitmask-vpn](https://github.com/leapcode/bitmask-vpn) | GPLv3 | Riseup/Calyx Road |
| psiphon-tunnel-core | [Psiphon-Inc/psiphon-tunnel-core](https://github.com/Psiphon-Inc/psiphon-tunnel-core) | GPLv3 | Road |
| Lokinet | [oxen-io/lokinet](https://github.com/oxen-io/lokinet) | GPL-3.0 | Overlay fallback |
| Tauri | [tauri-apps/tauri](https://github.com/tauri-apps/tauri) | Apache-2.0 / MIT | Desktop shell |

Pinned versions and hashes: `policy/engines.lock`.

DoH/DoT/DoQ and overlay **services** (Quad9, Cloudflare, DNS.SB, Control D, DNSBunker, Tor network, Riseup, Psiphon, Lokinet) are used as public networks. WNY does not rebrand them and does not proxy their terms.

No engine is modified except for generated config files written next to the local settings store.
