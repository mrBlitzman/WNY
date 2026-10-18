package org.wny.app

import android.content.Context
import org.json.JSONObject
import java.io.File

/** Local-only settings. Nothing leaves the device. */
class PolicyStore(private val ctx: Context) {
    private val file = File(ctx.filesDir, "settings.json")

    fun policy(): JSONObject = JSONObject(ctx.assets.open("policy.json").reader().readText())
    fun allowlist(): JSONObject = JSONObject(ctx.assets.open("allowlist.json").reader().readText())
    fun catalog(lang: String): JSONObject {
        val name = when (lang) {
            "tr" -> "tr.json"
            "de" -> "de.json"
            else -> "en.json"
        }
        return JSONObject(ctx.assets.open(name).reader().readText())
    }

    fun load(): JSONObject {
        if (!file.exists()) {
            return JSONObject()
                .put("on", false)
                .put("mode", "protection")
                .put("language", "en")
                .put("names", "auto")
                .put("filter", "auto")
                .put("road", "auto")
        }
        return JSONObject(file.readText())
    }

    fun save(s: JSONObject) {
        file.writeText(s.toString())
    }
}
