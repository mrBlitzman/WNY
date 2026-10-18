package hev.htproxy

/**
 * JNI contract for hev-socks5-tunnel AAR (bundled at release).
 * When the native library is absent, [available] is false and WNY fail-closes.
 */
object TProxyService {
    val available: Boolean = try {
        System.loadLibrary("hev-socks5-tunnel")
        true
    } catch (_: UnsatisfiedLinkError) {
        false
    }

    @JvmStatic external fun TProxyStartService(configPath: String, fd: Int): Boolean
    @JvmStatic external fun TProxyStopService(): Boolean
    @JvmStatic external fun TProxyIsRunning(): Boolean
}
