import Foundation
import NetworkExtension

/// Owns the one Packet Tunnel. iOS cannot run a second VPN at the same time.
final class WnyTunnelManager {
    static let shared = WnyTunnelManager()
    private init() {}

    func setEnabled(_ on: Bool, mode: String) async {
        do {
            let mgr = NEVPNManager.shared()
            try await mgr.loadFromPreferences()
            if on {
                let proto = NETunnelProviderProtocol()
                proto.providerBundleIdentifier = "org.wny.app.tunnel"
                proto.serverAddress = "WNY"
                proto.providerConfiguration = ["mode": mode]
                mgr.protocolConfiguration = proto
                mgr.localizedDescription = "WNY"
                mgr.isEnabled = true
                try await mgr.saveToPreferences()
                try mgr.connection.startVPNTunnel()
            } else {
                mgr.connection.stopVPNTunnel()
            }
        } catch {
            // Fail closed: leave the system disconnected rather than leak.
        }
    }
}
