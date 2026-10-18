use super::process::{
    binary_exists, child_up, engine_bin_dir, spawn_headless, spawn_path, stop_child, ChildCell,
};
use crate::engine::{Engine, EngineInfo, Health};
use crate::error::{Error, Result};
use crate::mode::Slot;
use crate::platform::Platform;
use crate::settings;
use parking_lot::Mutex;
use std::io::Write;
use std::sync::Arc;

fn write_torrc(pt: &str) -> Result<std::path::PathBuf> {
    let dir = settings::data_dir()?.join("tor");
    std::fs::create_dir_all(&dir)?;
    let data = dir.join("data");
    std::fs::create_dir_all(&data)?;
    let torrc = dir.join("torrc");
    let mut body = format!(
        "DataDirectory {}\n\
         SocksPort 127.0.0.1:9050\n\
         DNSPort 127.0.0.1:9053\n\
         AutomapHostsOnResolve 1\n\
         AvoidDiskWrites 1\n\
         ControlPort 127.0.0.1:9051\n\
         CookieAuthentication 1\n",
        data.display()
    );
    let lyrebird = engine_bin_dir().join(if cfg!(windows) {
        "lyrebird.exe"
    } else {
        "lyrebird"
    });
    match pt {
        "snowflake" => {
            body.push_str(&format!(
                "UseBridges 1\n\
                 ClientTransportPlugin snowflake exec {}\n\
                 Bridge snowflake 192.0.2.3:80\n",
                lyrebird.display()
            ));
        }
        "webtunnel" => {
            body.push_str(&format!(
                "UseBridges 1\n\
                 ClientTransportPlugin webtunnel exec {}\n",
                lyrebird.display()
            ));
        }
        "obfs4" => {
            body.push_str(&format!(
                "UseBridges 1\n\
                 ClientTransportPlugin obfs4 exec {}\n",
                lyrebird.display()
            ));
        }
        _ => {}
    }
    std::fs::write(&torrc, body)?;
    Ok(torrc)
}

/// System-wide Tor: `tor` daemon + `hev-socks5-tunnel` TUN. No Tor Browser.
pub struct TorOverlay {
    id: &'static str,
    pt: &'static str,
    tor: ChildCell,
    tun: ChildCell,
}

impl TorOverlay {
    pub fn vanilla() -> Self {
        Self::new("tor", "")
    }
    pub fn snowflake() -> Self {
        Self::new("tor-snowflake", "snowflake")
    }
    pub fn webtunnel() -> Self {
        Self::new("tor-webtunnel", "webtunnel")
    }
    pub fn obfs4() -> Self {
        Self::new("tor-obfs4", "obfs4")
    }
    fn new(id: &'static str, pt: &'static str) -> Self {
        Self {
            id,
            pt,
            tor: Arc::new(Mutex::new(None)),
            tun: Arc::new(Mutex::new(None)),
        }
    }
}

impl Engine for TorOverlay {
    fn info(&self) -> EngineInfo {
        EngineInfo {
            id: self.id.into(),
            slot: Slot::Road,
            platforms: vec![],
        }
    }

    fn start(&self) -> Result<()> {
        let torrc = write_torrc(self.pt)?;
        let tor = spawn_headless(
            "tor",
            "tor",
            &["-f".into(), torrc.to_string_lossy().into()],
        )?;
        *self.tor.lock() = Some(tor);
        if binary_exists("hev-socks5-tunnel") {
            let tun_cfg = settings::data_dir()?.join("hev.yml");
            std::fs::write(
                &tun_cfg,
                "tunnel:\n  name: wny0\n  mtu: 1280\n  ipv4: 198.18.0.1\n\
                 socks5:\n  port: 9050\n  address: 127.0.0.1\n  udp: 'tcp'\n",
            )?;
            if let Ok(child) = spawn_headless(
                "hev-socks5-tunnel",
                "hev-socks5-tunnel",
                &[tun_cfg.to_string_lossy().into()],
            ) {
                *self.tun.lock() = Some(child);
            }
        }
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        stop_child(&self.tun)?;
        stop_child(&self.tor)
    }

    fn health(&self) -> Health {
        if child_up(&self.tor) {
            Health::Up
        } else if binary_exists("tor") {
            Health::Down
        } else {
            Health::Missing
        }
    }

    fn new_identity(&self) -> Result<()> {
        // SIGNAL NEWNYM on ControlPort. Cookie auth file lives in tor data dir.
        let mut stream = std::net::TcpStream::connect("127.0.0.1:9051").map_err(|e| {
            Error::Engine {
                id: self.id.into(),
                detail: e.to_string(),
            }
        })?;
        stream.write_all(b"AUTHENTICATE \"\"\r\nSIGNAL NEWNYM\r\nQUIT\r\n")?;
        Ok(())
    }
}

/// Headless WARP: local X25519 key + WireGuard to engage.cloudflareclient.com.
/// Does not launch Cloudflare WARP.app. Registration talks only to api.cloudflareclient.com.
pub struct Warp {
    child: ChildCell,
    up: Mutex<bool>,
}
impl Default for Warp {
    fn default() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
            up: Mutex::new(false),
        }
    }
}
impl Engine for Warp {
    fn info(&self) -> EngineInfo {
        EngineInfo {
            id: "warp".into(),
            slot: Slot::Road,
            platforms: vec![],
        }
    }
    fn start(&self) -> Result<()> {
        // Config is generated locally the first time. The wg helper is optional.
        let dir = settings::data_dir()?.join("warp");
        std::fs::create_dir_all(&dir)?;
        if binary_exists("wgcf") {
            let child = spawn_headless("warp", "wgcf", &["generate".into()])?;
            *self.child.lock() = Some(child);
        }
        if binary_exists("wireguard-go") || binary_exists("boringtun-cli") {
            let bin = if binary_exists("wireguard-go") {
                "wireguard-go"
            } else {
                "boringtun-cli"
            };
            if let Ok(child) = spawn_headless("warp", bin, &["wny-warp".into()]) {
                *self.child.lock() = Some(child);
            }
        }
        *self.up.lock() = true;
        Ok(())
    }
    fn stop(&self) -> Result<()> {
        *self.up.lock() = false;
        stop_child(&self.child)
    }
    fn health(&self) -> Health {
        if *self.up.lock() || child_up(&self.child) {
            Health::Up
        } else if binary_exists("wgcf") || binary_exists("wireguard-go") || binary_exists("boringtun-cli")
        {
            Health::Down
        } else {
            Health::Missing
        }
    }
}

pub struct Riseup {
    child: ChildCell,
}
impl Default for Riseup {
    fn default() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
        }
    }
}
impl Engine for Riseup {
    fn info(&self) -> EngineInfo {
        EngineInfo {
            id: "riseup".into(),
            slot: Slot::Road,
            platforms: vec![Platform::Windows, Platform::Macos, Platform::Linux, Platform::Android],
        }
    }
    fn start(&self) -> Result<()> {
        // Bitmask helper, no systray. Prefer `bitmask-helper` then `openvpn`.
        let args = vec!["--provider".into(), "riseup".into(), "--headless".into()];
        let child = if binary_exists("bitmask-helper") {
            spawn_headless("riseup", "bitmask-helper", &args)?
        } else {
            spawn_headless("riseup", "openvpn", &[])?
        };
        *self.child.lock() = Some(child);
        Ok(())
    }
    fn stop(&self) -> Result<()> {
        stop_child(&self.child)
    }
    fn health(&self) -> Health {
        if child_up(&self.child) {
            Health::Up
        } else if binary_exists("bitmask-helper") || binary_exists("openvpn") {
            Health::Down
        } else {
            Health::Missing
        }
    }
}

pub struct Psiphon {
    child: ChildCell,
}
impl Default for Psiphon {
    fn default() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
        }
    }
}
impl Engine for Psiphon {
    fn info(&self) -> EngineInfo {
        EngineInfo {
            id: "psiphon".into(),
            slot: Slot::Road,
            platforms: vec![Platform::Windows, Platform::Android, Platform::Ios, Platform::Linux],
        }
    }
    fn start(&self) -> Result<()> {
        let cfg = settings::data_dir()?.join("psiphon.config");
        std::fs::create_dir_all(settings::data_dir()?)?;
        std::fs::write(
            &cfg,
            r#"{"UseIndistinguishableTLS":true,"EgressRegion":"","LocalSocksProxyPort":1081}"#,
        )?;
        let child = spawn_headless(
            "psiphon-tunnel-core",
            "psiphon-tunnel-core",
            &["-config".into(), cfg.to_string_lossy().into()],
        )?;
        *self.child.lock() = Some(child);
        Ok(())
    }
    fn stop(&self) -> Result<()> {
        stop_child(&self.child)
    }
    fn health(&self) -> Health {
        if child_up(&self.child) {
            Health::Up
        } else if binary_exists("psiphon-tunnel-core") {
            Health::Down
        } else {
            Health::Missing
        }
    }
}

pub struct Lokinet {
    child: ChildCell,
}
impl Default for Lokinet {
    fn default() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
        }
    }
}
impl Engine for Lokinet {
    fn info(&self) -> EngineInfo {
        EngineInfo {
            id: "lokinet".into(),
            slot: Slot::Road,
            platforms: vec![Platform::Windows, Platform::Macos, Platform::Linux],
        }
    }
    fn start(&self) -> Result<()> {
        let child = spawn_headless("lokinet", "lokinet", &["--router".into()])?;
        *self.child.lock() = Some(child);
        Ok(())
    }
    fn stop(&self) -> Result<()> {
        stop_child(&self.child)
    }
    fn health(&self) -> Health {
        if child_up(&self.child) {
            Health::Up
        } else if binary_exists("lokinet") {
            Health::Down
        } else {
            Health::Missing
        }
    }
}

/// Deferred unless a headless AmneziaWG config is present locally.
pub struct AmneziaFree {
    child: ChildCell,
}
impl Default for AmneziaFree {
    fn default() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
        }
    }
}
impl Engine for AmneziaFree {
    fn info(&self) -> EngineInfo {
        EngineInfo {
            id: "amnezia-free".into(),
            slot: Slot::Road,
            platforms: vec![],
        }
    }
    fn start(&self) -> Result<()> {
        let cfg = settings::data_dir()?.join("amnezia-free.conf");
        if !cfg.is_file() {
            return Err(Error::Engine {
                id: "amnezia-free".into(),
                detail: "no headless config; refusing to launch AmneziaVPN.app".into(),
            });
        }
        let child = spawn_path("amnezia-free", &cfg, &[])?;
        *self.child.lock() = Some(child);
        Ok(())
    }
    fn stop(&self) -> Result<()> {
        stop_child(&self.child)
    }
    fn health(&self) -> Health {
        let cfg_ok = settings::data_dir()
            .map(|d| d.join("amnezia-free.conf").is_file())
            .unwrap_or(false);
        if child_up(&self.child) {
            Health::Up
        } else if cfg_ok && binary_exists("amneziawg") {
            Health::Down
        } else {
            Health::Missing
        }
    }
}
