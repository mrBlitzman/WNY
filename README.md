# WNY (WholeNineYards)

WNY is a single switch for the whole device. It encrypts DNS, blocks unsafe (MITM) connections, bypasses regional filtering when it can, and can send **all apps** through a hidden overlay network. You keep using Chrome, Safari, WhatsApp, and mail as usual. WNY never opens Tor Browser or another VPN window.

Free. No account. Settings stay on this device.

When WNY is **on**, unencrypted packets are not allowed out. WNY talks only to the networks listed in [ARCHITECTURE.md](ARCHITECTURE.md).

## Install

### Windows

1. Download `WNY-windows.msi` from [Releases](../../releases).
2. Run the installer. Allow the VPN / driver prompt when asked.
3. Choose language (English, Türkçe, Deutsch).
4. Flip the switch.

### macOS

1. Download `WNY-macos.dmg`.
2. Drag WNY to Applications. Grant VPN / network filter when macOS asks.
3. Flip the switch.

### Android

1. Install the APK from Releases (Play is optional later).
2. Accept the VPN permission. Enable always-on / Block connections without VPN when WNY offers it.
3. Flip the switch.

### iOS

1. Install WNY from the App Store or a signed build.
2. Allow adding a VPN configuration. iOS cannot do local DPI; Unblock uses a tunnel, Private uses Tor inside WNY.
3. Flip the switch.

First launch is a short wizard: language → what WNY does → permissions → optional start at login → done. After that, daily use does not change.

## Modes

- **Protection** — this network is hostile (cafe Wi-Fi). Names are encrypted. MITM is blocked.
- **Unblock** — sites are filtered. WNY tries local bypass, then a tunnel, then a hidden network.
- **Private** — hide the device address. All traffic goes through Tor (or the next overlay). Some calls and games will pause.
- **Emergency** — try every path. If nothing works, internet stays off rather than leak.

The main screen always shows which tool is in **Names**, **Filter**, and **Road**. Auto failovers unless you pin a tool.

## Build from source

Requirements: Rust 1.83+, Node 22+ (desktop), Android SDK (Android), Xcode (iOS).

```bash
# policy + core tests (any OS)
cargo test --workspace --exclude wny-desktop

# optional: fetch pinned engine binaries (maintainer)
./scripts/fetch-engines.sh

# desktop (Linux/macOS/Windows)
cd apps/desktop && npm install && npm run tauri build
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for the allowlist, layers, and threat model. Third-party licenses are in [THIRD_PARTY.md](THIRD_PARTY.md).

## What WNY is not

It is not an anonymity guarantee for signed-in accounts. It is not a custom VPN protocol. It will not send your settings anywhere.
