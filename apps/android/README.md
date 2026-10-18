# WNY Android

`VpnService` with always-on lockdown. Filter is ByeDPI inside the same VPN. Overlay is Arti/onionmasq when bundled, otherwise `tor` + hev-socks5-tunnel (`hev.htproxy.TProxyService`). Settings stay in `filesDir/settings.json`.

```
cd apps/android
# open in Android Studio or:
./gradlew :app:assembleDebug   # add the Gradle wrapper on a machine with Android SDK
```

Enable **Block connections without VPN** when WNY asks. That is the kill-switch.
