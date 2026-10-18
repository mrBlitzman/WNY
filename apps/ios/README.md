# WNY iOS

One app, one Packet Tunnel. No Filter slot (the UI says so). Road is WARP or embedded Tor. The 50 MB Network Extension memory limit means pluggable transports may be skipped; Unblock then uses WARP/Psiphon inside the same extension.

Build on macOS:

```
brew install xcodegen
cd apps/ios && xcodegen generate
open WNY.xcodeproj
```

Bundle `tor` / `hev-socks5-tunnel` / WireGuardKit inside the tunnel target. Do not ship Onion Browser or Orbot.
