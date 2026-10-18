# WNY Architecture

WNY (WholeNineYards) is a local orchestrator. It does not implement VPN, Tor, DPI, or DNS protocols. It installs, starts, health-checks, and failovers existing free, no-account engines behind one UI.

When the switch is **on**, every user packet is captured by the WNY TUN/VPN adapter, then classified by L0, then passed through three slots: **Names**, **Filter**, **Road**. Daily apps do not change. No third-party UI is opened.

WNY has no first-party backend. The only persistent data is on-device settings and keys generated locally.

## L0 — Self-security (always on with the switch)

| Rule | Behavior |
| --- | --- |
| Local only | Settings and keys live in OS app-data. No account, cloud, crash reporter, analytics, or off-device install ID. |
| No plaintext egress | Drop TCP/UDP 53, TCP 80, and any flow that is not TLS, WireGuard, Tor, DoH, DoT, or DoQ. |
| Allowlist only | WNY and child engines may reach only destinations classified in this file. Everything else is dropped. |
| No extra services | No updater daemon. Updates are user-initiated from the GitHub release URL listed below. |
| Integrity | Bundled engine bytes must match `policy/engines.lock`. Tamper → refuse to start. |
| Logs | In-memory ring by default. Destination hostnames/IPs are not written to disk. |

Captive portal is **not** automatic. The control **Sign into this Wi-Fi** opens a timed plaintext window, then relocks. That is the only documented plaintext exception.

Private mode also drops UDP/443 (QUIC) and STUN/TURN so WebRTC cannot leak a real address.

## Traffic path

```
User apps (unchanged)
        │
        ▼
WNY TUN / VpnService / Packet Tunnel
        │
        ▼
L0 self-security (plaintext + non-allowlist → drop)
        │
        ▼
Names (encrypted DNS) → Filter (local DPI) → Road (tunnel or overlay)
```

iOS has no Filter slot. Android uses always-on VPN lockdown. Desktop uses WFP (Windows) or pf (macOS).

## Modes

| Mode | Names | Filter | Road | Intent |
| --- | --- | --- | --- | --- |
| Protection | Encrypted DNS | Off unless DPI/MITM | Off unless MITM → WARP | Hostile Wi-Fi, no identity claim |
| Unblock | Auto DNS | Auto DPI | Auto tunnel, then overlay | Reach blocked sites |
| Private | Overlay DNS only | Only if Tor cannot bootstrap | Overlay required | Hide network identity |
| Emergency | Auto | Auto | Auto, no pins | Try every path; still fail-closed |

Pinned engines never silent-failover. If a pin dies, traffic stops and the UI asks to switch.

## Slots and engines

### Names (encrypted DNS)

Order: Quad9 → Cloudflare → DNS.SB. Spare: Control D uncensored, DNSBunker, ODoH (Protection/Unblock only).

Engine: `dnscrypt-proxy`, configured by WNY. Private mode uses Tor `DNSPort` instead of public DoH.

### Filter (local DPI, not on iOS)

| Platform | Order |
| --- | --- |
| Windows | GoodbyeDPI → zapret winws → ByeDPI |
| macOS | SpoofDPI → zapret tpws → ByeDPI |
| Android | ByeDPI inside WNY VpnService → PowerTunnel-class fallback |
| iOS | none |

Filter does not change IP. It only desynchronizes ISP DPI.

### Road

Unblock order: WARP (headless WireGuard) → Riseup/Calyx (Bitmask helpers, no tray) → Psiphon tunnel-core → overlay.

Private order: Tor TUN → Filter as Tor entry assist → Snowflake → WebTunnel/obfs4 → Lokinet → **fail closed**.

Overlay is system-wide (`tor` expert bundle + `hev-socks5-tunnel`, or Arti/onionmasq on Android). Tor Browser, Onion Browser, and CENO are not shipped.

## Allowlist

WNY must not talk to any host not classified here. Dynamic networks are named classes, not IP enumerations.

### Static — DNS

| Name | Bootstrap | Encrypted endpoint |
| --- | --- | --- |
| Quad9 | `9.9.9.9`, `149.112.112.112`, `2620:fe::fe`, `2620:fe::9` | `https://dns.quad9.net/dns-query`, DoT `dns.quad9.net:853` |
| Cloudflare | `1.1.1.1`, `1.0.0.1`, `2606:4700:4700::1111`, `2606:4700:4700::1001` | `https://cloudflare-dns.com/dns-query` |
| DNS.SB | `185.222.222.222`, `45.11.45.11` | `https://doh.dns.sb/dns-query` |
| Control D uncensored | `76.76.2.5`, `76.76.10.5` | `https://freedns.controld.com/uncensored` |
| DNSBunker | (encrypted only) | `https://dnsbunker.org/dns-query` |
| ODoH | Cloudflare | `https://odoh.cloudflare-dns.com/dns-query` |

DoH host IPs are pinned so the first lookup never uses ISP DNS.

### Static — WARP

- `https://api.cloudflareclient.com` (device registration; X25519 private key never leaves the device)
- WireGuard peer `engage.cloudflareclient.com:2408` and Cloudflare anycast `162.159.192.0/24`, `162.159.193.0/24`, `2606:4700:d0::/48`

Official WARP.app is not launched.

### Static — Riseup / Calyx (Bitmask)

Documented provider API hosts used by [leapcode/bitmask-vpn](https://github.com/leapcode/bitmask-vpn):

- `api.black.riseup.net`
- `api.riseup.net`
- `calyx.net` / Bitmask provider JSON for Calyx

Helpers run headless. RiseupVPN systray is not shown.

### Named classes (dynamic by design)

| Class | Who may use it | What it is |
| --- | --- | --- |
| `tor-authorities` | `tor` process | Tor directory authorities from the expert bundle |
| `tor-relays` | `tor` process | Guards, middles, exits learned from consensus |
| `tor-bridges` | `tor` + lyrebird | Bundled obfs4 / WebTunnel / meek bridges |
| `snowflake` | lyrebird | Snowflake broker + volunteer proxies (WebRTC) |
| `psiphon-network` | `psiphon-tunnel-core` | Official bootstrap + learned servers |
| `lokinet-dht` | `lokinet` | Service nodes and `exit.loki` |
| `amnezia-free` | AmneziaWG helper | Only if a headless config is obtainable without their GUI |

User apps never speak these classes directly. Only the active engine process does, after L0 has attached the TUN.

### Build / update only (not while protecting)

- GitHub Releases for this repository (user-initiated WNY update)
- `https://www.torproject.org/download/tor/` and `https://dist.torproject.org/` (maintainer `scripts/fetch-engines.sh`)
- Upstream GitHub Releases listed in `policy/engines.lock`

Runtime protection path must not depend on GitHub.

## Forbidden

Proton VPN, Windscribe, NymVPN (account or payment), public Xray/Hysteria/Reality lists, third-party WARP config websites, NextDNS profiles, Mullvad DNS (discontinued 2026-11-02), Yggdrasil, Tor Browser / Onion Browser / CENO as user-facing apps, WNY installing its own TLS CA.

## Identity (Private)

1. **Network person** — multi-hop overlay IP (Tor, then PTs, then Lokinet).
2. **Unlink** — Android: per-app circuits. Desktop: `IsolateDestAddr` + NEWNYM. iOS: shared tunnel, UI says so.
3. **Leak shield** — drop IPv6 if the overlay does not carry it, STUN/TURN, mDNS/LLMNR/NetBIOS, QUIC, WebRTC host candidates.

Signed-in apps still know the user. WNY states that in one sentence. It does not log anyone out.

## Probes

Canaries are compiled in (`policy/policy.json`), not user-edited:

- DNS poison: DoH vs plaintext 53 for `wikipedia.org` / `quad9.net` (plaintext 53 is blocked while ON; probe runs in a controlled exception or before engage).
- TLS MITM: leaf issuer of `github.com` / `cloudflare.com` against public CA + CT set.
- User-added root CA inventory.
- Connectivity: small site set for Unblock.
- Leaks: IPv6, DNS whoami, WebRTC.

Probe traffic itself uses jitter and a tiny set so the checker is not a fingerprint.

## Platforms

| | Windows | macOS | Android | iOS |
| --- | --- | --- | --- | --- |
| Shell | Tauri | Tauri | Kotlin | SwiftUI |
| Capture | WinTUN + hev-socks5-tunnel | utun + hev | VpnService | Packet Tunnel |
| L0 | WFP | pf | always-on lockdown | NE drop |
| Overlay | tor + lyrebird | same | Arti / onionmasq or tor+hev | embedded Tor (50 MB NE limit) |

One visible app. Child engines are hidden processes.

## Settings location

- Windows: `%AppData%/WNY/settings.json` and key files
- macOS: `~/Library/Application Support/WNY/`
- Android / iOS: app sandbox

Keys (WARP private key, Tor state) never leave the device.
