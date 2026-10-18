use crate::mode::Mode;
use crate::policy::Policy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
    Tcp,
    Udp,
    Icmp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PayloadKind {
    /// TLS 1.2/1.3 ClientHello / ApplicationData
    Tls,
    /// DNS-over-HTTPS / HTTP/2 over TLS (still TLS on the wire)
    Doh,
    /// WireGuard handshake or transport
    Wireguard,
    /// Tor cell / OR protocol on its ORPort (TLS in practice for modern Tor)
    Tor,
    /// DNS-over-TLS
    Dot,
    /// DNS-over-QUIC — still encrypted, but Private drops QUIC
    Doq,
    /// Plain DNS, HTTP/1.x, STUN, mDNS, etc.
    Clear,
    Other,
}

#[derive(Debug, Clone, Copy)]
pub struct Flow {
    pub transport: Transport,
    pub dport: u16,
    pub kind: PayloadKind,
}

/// Returns true if this flow must be dropped while WNY is on.
pub fn is_plaintext(flow: Flow, mode: Mode, captive_portal: bool, policy: &Policy) -> bool {
    if captive_portal {
        return false;
    }
    if policy.plaintext_drop.tcp_ports.contains(&flow.dport) && flow.transport == Transport::Tcp {
        return true;
    }
    if policy.plaintext_drop.udp_ports.contains(&flow.dport) && flow.transport == Transport::Udp {
        return true;
    }
    if mode == Mode::Private
        && policy.plaintext_drop.private_also_drop_udp_443
        && flow.transport == Transport::Udp
        && flow.dport == 443
    {
        return true;
    }
    match flow.kind {
        PayloadKind::Tls | PayloadKind::Doh | PayloadKind::Wireguard | PayloadKind::Tor | PayloadKind::Dot => {
            false
        }
        PayloadKind::Doq => mode == Mode::Private,
        PayloadKind::Clear => true,
        PayloadKind::Other => {
            // Unknown UDP that is not a known encrypted port is dropped.
            flow.transport != Transport::Tcp || flow.dport != 443
        }
    }
}

/// First-byte heuristic for the userspace path (TUN copy). Conservative: if we
/// cannot prove encryption, treat as clear.
pub fn classify_payload(transport: Transport, dport: u16, head: &[u8]) -> PayloadKind {
    if dport == 53 {
        return PayloadKind::Clear;
    }
    if dport == 80 && transport == Transport::Tcp {
        return PayloadKind::Clear;
    }
    if head.len() >= 1 && head[0] == 0x16 {
        // TLS record type handshake
        return PayloadKind::Tls;
    }
    if head.len() >= 1 && head[0] == 0x17 {
        return PayloadKind::Tls;
    }
    // WireGuard: 1..=4 message types in first 4 little-endian bytes, rest zeros in type field
    if transport == Transport::Udp && head.len() >= 4 {
        let t = u32::from_le_bytes(head[0..4].try_into().unwrap());
        if (1..=4).contains(&t) {
            return PayloadKind::Wireguard;
        }
    }
    if dport == 443 && transport == Transport::Tcp {
        return PayloadKind::Tls;
    }
    if dport == 853 {
        return PayloadKind::Dot;
    }
    PayloadKind::Clear
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::Policy;

    fn pol() -> Policy {
        Policy::bundled().unwrap()
    }

    #[test]
    fn drops_dns_and_http() {
        let p = pol();
        let dns = Flow {
            transport: Transport::Udp,
            dport: 53,
            kind: PayloadKind::Clear,
        };
        assert!(is_plaintext(dns, Mode::Protection, false, &p));
        let http = Flow {
            transport: Transport::Tcp,
            dport: 80,
            kind: PayloadKind::Clear,
        };
        assert!(is_plaintext(http, Mode::Unblock, false, &p));
        assert!(!is_plaintext(http, Mode::Unblock, true, &p));
    }

    #[test]
    fn allows_tls_and_wg() {
        let p = pol();
        let tls = Flow {
            transport: Transport::Tcp,
            dport: 443,
            kind: PayloadKind::Tls,
        };
        assert!(!is_plaintext(tls, Mode::Protection, false, &p));
        let wg = Flow {
            transport: Transport::Udp,
            dport: 2408,
            kind: PayloadKind::Wireguard,
        };
        assert!(!is_plaintext(wg, Mode::Unblock, false, &p));
    }

    #[test]
    fn private_drops_quic() {
        let p = pol();
        let quic = Flow {
            transport: Transport::Udp,
            dport: 443,
            kind: PayloadKind::Doq,
        };
        assert!(is_plaintext(quic, Mode::Private, false, &p));
        assert!(!is_plaintext(quic, Mode::Unblock, false, &p));
    }

    #[test]
    fn classify_tls_handshake() {
        assert_eq!(
            classify_payload(Transport::Tcp, 443, &[0x16, 0x03, 0x03]),
            PayloadKind::Tls
        );
        assert_eq!(
            classify_payload(Transport::Udp, 53, b"\x12\x34"),
            PayloadKind::Clear
        );
    }
}
