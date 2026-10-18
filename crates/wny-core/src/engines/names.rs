use super::process::{binary_exists, child_up, spawn_headless, stop_child, ChildCell};
use crate::engine::{Engine, EngineInfo, Health};
use crate::error::Result;
use crate::mode::Slot;
use crate::platform::Platform;
use crate::settings;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct NamesEngine {
    id: &'static str,
    server_name: &'static str,
    child: ChildCell,
}

impl NamesEngine {
    fn new(id: &'static str, server_name: &'static str) -> Self {
        Self {
            id,
            server_name,
            child: Arc::new(Mutex::new(None)),
        }
    }

    fn write_config(&self) -> Result<std::path::PathBuf> {
        let dir = settings::data_dir()?.join("dnscrypt");
        std::fs::create_dir_all(&dir)?;
        let cfg = dir.join(format!("{}.toml", self.id));
        let body = format!(
            "listen_addresses = ['127.0.0.1:53']\n\
             server_names = ['{server}']\n\
             ipv6_servers = false\n\
             block_ipv6 = true\n",
            server = self.server_name
        );
        std::fs::write(&cfg, body)?;
        Ok(cfg)
    }
}

impl Engine for NamesEngine {
    fn info(&self) -> EngineInfo {
        EngineInfo {
            id: self.id.into(),
            slot: Slot::Names,
            platforms: vec![
                Platform::Windows,
                Platform::Macos,
                Platform::Linux,
                Platform::Android,
            ],
        }
    }

    fn start(&self) -> Result<()> {
        let cfg = self.write_config()?;
        let child = spawn_headless(
            "dnscrypt-proxy",
            "dnscrypt-proxy",
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
        } else if binary_exists("dnscrypt-proxy") {
            Health::Down
        } else {
            Health::Missing
        }
    }
}

macro_rules! dns_engine {
    ($t:ident, $id:expr, $server:expr) => {
        pub struct $t {
            inner: NamesEngine,
        }
        impl Default for $t {
            fn default() -> Self {
                Self {
                    inner: NamesEngine::new($id, $server),
                }
            }
        }
        impl Engine for $t {
            fn info(&self) -> EngineInfo {
                self.inner.info()
            }
            fn start(&self) -> Result<()> {
                self.inner.start()
            }
            fn stop(&self) -> Result<()> {
                self.inner.stop()
            }
            fn health(&self) -> Health {
                self.inner.health()
            }
        }
    };
}

dns_engine!(Quad9, "quad9", "quad9-dnscrypt-ip4-filter-pri");
dns_engine!(CloudflareDns, "cloudflare", "cloudflare");
dns_engine!(DnsSb, "dns-sb", "dnssb");
dns_engine!(ControlD, "controld-uncensored", "controld-uncensored");
dns_engine!(DnsBunker, "dnsbunker", "dnsbunker");
dns_engine!(Odoh, "odoh", "odoh-cloudflare");

/// Private-mode DNS: Tor DNSPort. The Tor overlay engine owns the process;
/// this engine is a marker so the Names slot can display "Hidden DNS".
pub struct TorDns {
    up: Mutex<bool>,
}
impl Default for TorDns {
    fn default() -> Self {
        Self {
            up: Mutex::new(false),
        }
    }
}
impl Engine for TorDns {
    fn info(&self) -> EngineInfo {
        EngineInfo {
            id: "tor-dns".into(),
            slot: Slot::Names,
            platforms: vec![],
        }
    }
    fn start(&self) -> Result<()> {
        *self.up.lock() = true;
        Ok(())
    }
    fn stop(&self) -> Result<()> {
        *self.up.lock() = false;
        Ok(())
    }
    fn health(&self) -> Health {
        if *self.up.lock() {
            Health::Up
        } else if binary_exists("tor") {
            Health::Down
        } else {
            Health::Missing
        }
    }
}
