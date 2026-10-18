package org.wny.app

import android.app.Activity
import android.content.Intent
import android.net.VpnService
import android.os.Bundle
import android.widget.ArrayAdapter
import android.widget.Button
import android.widget.Spinner
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import org.json.JSONObject

class MainActivity : AppCompatActivity() {
    private lateinit var store: PolicyStore
    private lateinit var settings: JSONObject
    private lateinit var cat: JSONObject

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
        store = PolicyStore(this)
        settings = store.load()
        bind()
        findViewById<Button>(R.id.power).setOnClickListener { toggle() }
        findViewById<Button>(R.id.ident).setOnClickListener {
            // Tor NEWNYM is issued by the overlay process; UI stays in WNY.
        }
    }

    private fun bind() {
        val lang = settings.optString("language", "en")
        cat = store.catalog(lang)
        findViewById<TextView>(R.id.status).text =
            if (settings.optBoolean("on")) cat.getJSONObject("status").optString("protecting")
            else cat.getJSONObject("status").optString("idle")
        findViewById<Button>(R.id.power).text =
            if (settings.optBoolean("on")) cat.optString("on") else cat.optString("off")
        val modes = listOf("protection", "unblock", "private", "emergency")
        val labels = modes.map { cat.getJSONObject("modes").optString(it) }
        val spinner = findViewById<Spinner>(R.id.mode)
        spinner.adapter = ArrayAdapter(this, android.R.layout.simple_spinner_dropdown_item, labels)
        spinner.setSelection(modes.indexOf(settings.optString("mode", "protection")).coerceAtLeast(0))
        val slots = cat.getJSONObject("slots")
        findViewById<TextView>(R.id.namesLine).text =
            "${slots.optString("names")}: ${settings.optString("names", "auto")}"
        findViewById<TextView>(R.id.filterLine).text =
            "${slots.optString("filter")}: ${settings.optString("filter", "auto")} · ByeDPI"
        findViewById<TextView>(R.id.roadLine).text =
            "${slots.optString("road")}: ${settings.optString("road", "auto")} · Tor"
        findViewById<Button>(R.id.ident).text = cat.getJSONObject("actions").optString("newIdentity")
    }

    private fun toggle() {
        val next = !settings.optBoolean("on")
        if (next) {
            val prep = VpnService.prepare(this)
            if (prep != null) {
                startActivityForResult(prep, 7)
                return
            }
            startVpn()
        } else {
            startService(Intent(this, WnyVpnService::class.java).setAction(WnyVpnService.ACTION_STOP))
        }
        settings.put("on", next)
        store.save(settings)
        bind()
    }

    private fun startVpn() {
        startForegroundService(Intent(this, WnyVpnService::class.java))
    }

    @Deprecated("prepare callback")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (requestCode == 7 && resultCode == Activity.RESULT_OK) {
            startVpn()
            settings.put("on", true)
            store.save(settings)
            bind()
        }
    }
}
