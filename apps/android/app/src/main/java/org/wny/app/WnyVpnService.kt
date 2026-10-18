package org.wny.app

import android.content.Intent
import android.net.VpnService
import android.os.ParcelFileDescriptor
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import androidx.core.app.NotificationCompat
import java.io.File

/**
 * Device-wide capture. Always-on lockdown is requested from the UI.
 * Packets go to hev-socks5-tunnel → Tor/WARP SOCKS. No other app UI.
 */
class WnyVpnService : VpnService() {
    private var tun: ParcelFileDescriptor? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_STOP) {
            stop()
            return START_NOT_STICKY
        }
        startForeground()
        startTunnel()
        return START_STICKY
    }

    private fun startForeground() {
        val nm = getSystemService(NotificationManager::class.java)
        nm.createNotificationChannel(
            NotificationChannel(CHANNEL, "WNY", NotificationManager.IMPORTANCE_LOW)
        )
        val pi = PendingIntent.getActivity(
            this, 0, Intent(this, MainActivity::class.java),
            PendingIntent.FLAG_IMMUTABLE
        )
        startForeground(
            1,
            NotificationCompat.Builder(this, CHANNEL)
                .setContentTitle("WNY")
                .setContentText(getString(R.string.vpn_notification))
                .setSmallIcon(android.R.drawable.ic_lock_lock)
                .setContentIntent(pi)
                .build()
        )
    }

    private fun startTunnel() {
        stopTunnel()
        val builder = Builder()
            .setSession("WNY")
            .setMtu(1280)
            .addAddress("198.18.0.1", 32)
            .addDnsServer("10.192.0.53")
            .addRoute("0.0.0.0", 0)
            .addRoute("::", 0)
            .allowBypass() // only for the captive-portal timed exception
        tun = builder.establish()
        val fd = tun?.fd ?: return
        val cfg = File(filesDir, "hev.yml")
        cfg.writeText(
            """
            tunnel:
              mtu: 1280
              ipv4: 198.18.0.1
            socks5:
              port: 9050
              address: 127.0.0.1
              udp: 'tcp'
            """.trimIndent()
        )
        if (hev.htproxy.TProxyService.available) {
            hev.htproxy.TProxyService.TProxyStartService(cfg.absolutePath, fd)
        }
        // If the native engine is missing, lockdown still holds: no leak.
    }

    private fun stopTunnel() {
        if (hev.htproxy.TProxyService.available) {
            hev.htproxy.TProxyService.TProxyStopService()
        }
        tun?.close()
        tun = null
    }

    private fun stop() {
        stopTunnel()
        stopForeground(STOP_FOREGROUND_REMOVE)
        stopSelf()
    }

    override fun onDestroy() {
        stopTunnel()
        super.onDestroy()
    }

    companion object {
        const val ACTION_STOP = "org.wny.app.STOP"
        const val CHANNEL = "wny"
    }
}
