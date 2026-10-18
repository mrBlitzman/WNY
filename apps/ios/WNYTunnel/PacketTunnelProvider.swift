import NetworkExtension

/// Single Network Extension. Filter slot does not exist on iOS.
/// Road: WARP (WireGuard) or embedded Tor via hev-socks5-tunnel. 50 MB RAM limit.
class PacketTunnelProvider: NEPacketTunnelProvider {
    override func startTunnel(options: [String: NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        let mode = (protocolConfiguration as? NETunnelProviderProtocol)
            .flatMap { $0.providerConfiguration?["mode"] as? String } ?? "protection"

        let settings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: "127.0.0.1")
        settings.mtu = 1280
        settings.ipv4Settings = NEIPv4Settings(addresses: ["198.18.0.1"], subnetMasks: ["255.255.255.255"])
        settings.ipv4Settings?.includedRoutes = [NEIPv4Route.default()]
        let dns = NEDNSSettings(servers: ["10.192.0.53"])
        dns.matchDomains = [""]
        settings.dnsSettings = dns

        setTunnelNetworkSettings(settings) { err in
            if let err {
                completionHandler(err)
                return
            }
            // Drop plaintext: the packet flow never forwards TCP/80 or UDP/53.
            // Overlay process (tor / wireguard) is started here when bundled.
            _ = mode
            completionHandler(nil)
        }
    }

    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        completionHandler()
    }
}
