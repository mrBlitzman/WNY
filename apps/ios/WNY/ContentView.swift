import SwiftUI
import NetworkExtension

/// Single visible UI. No Tor Browser, Orbot, or WARP.app.
struct ContentView: View {
    @AppStorage("wny.on") private var on = false
    @AppStorage("wny.mode") private var mode = "protection"
    @AppStorage("wny.lang") private var lang = "en"

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("WNY").font(.largeTitle.weight(.semibold))
            Text(statusText).foregroundStyle(.secondary)
            Button(on ? label("on") : label("off")) { toggle() }
                .buttonStyle(.borderedProminent)
                .frame(maxWidth: .infinity, minHeight: 72)
            Picker(label("mode"), selection: $mode) {
                Text(label("modes.protection")).tag("protection")
                Text(label("modes.unblock")).tag("unblock")
                Text(label("modes.private")).tag("private")
                Text(label("modes.emergency")).tag("emergency")
            }
            slot("slots.names", engine: mode == "private" ? "Hidden DNS" : "Quad9")
            Text(label("status.iosNoFilter")).font(.footnote).foregroundStyle(.secondary)
            slot("slots.road", engine: mode == "private" ? "Tor" : "WARP")
            Spacer()
        }
        .padding()
        .preferredColorScheme(.dark)
    }

    private var statusText: String {
        if !on { return label("status.idle") }
        if mode == "private" { return label("status.private") + " " + label("status.iosSharedPath") }
        return label("status.protecting")
    }

    private func slot(_ key: String, engine: String) -> some View {
        VStack(alignment: .leading) {
            Text(label(key)).font(.caption).foregroundStyle(.secondary)
            Text(engine).font(.headline)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding()
        .background(Color.white.opacity(0.06), in: RoundedRectangle(cornerRadius: 12))
    }

    private func toggle() {
        on.toggle()
        Task { await WnyTunnelManager.shared.setEnabled(on, mode: mode) }
    }

    private func label(_ path: String) -> String {
        Catalog.string(lang: lang, path: path)
    }
}

enum Catalog {
    static func string(lang: String, path: String) -> String {
        let parts = path.split(separator: ".").map(String.init)
        guard let url = Bundle.main.url(forResource: lang, withExtension: "json"),
              let data = try? Data(contentsOf: url),
              var obj = try? JSONSerialization.jsonObject(with: data) else {
            return path
        }
        for p in parts {
            if let d = obj as? [String: Any] { obj = d[p] as Any }
        }
        return obj as? String ?? path
    }
}
