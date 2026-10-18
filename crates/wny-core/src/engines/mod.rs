mod filter;
mod names;
mod process;
mod road;

pub use filter::*;
pub use names::*;
pub use process::{binary_exists, engine_bin_dir, spawn_headless, stop_child, ChildCell};
pub use road::*;

use crate::engine::Engine;
use crate::platform::Platform;
use std::sync::Arc;

pub fn all() -> Vec<Arc<dyn Engine>> {
    let mut v: Vec<Arc<dyn Engine>> = vec![
        Arc::new(Quad9::default()),
        Arc::new(CloudflareDns::default()),
        Arc::new(DnsSb::default()),
        Arc::new(ControlD::default()),
        Arc::new(DnsBunker::default()),
        Arc::new(Odoh::default()),
        Arc::new(TorDns::default()),
        Arc::new(GoodbyeDpi::default()),
        Arc::new(Zapret::default()),
        Arc::new(SpoofDpi::default()),
        Arc::new(ByeDpi::default()),
        Arc::new(Warp::default()),
        Arc::new(Riseup::default()),
        Arc::new(Psiphon::default()),
        Arc::new(TorOverlay::vanilla()),
        Arc::new(TorOverlay::snowflake()),
        Arc::new(TorOverlay::webtunnel()),
        Arc::new(TorOverlay::obfs4()),
        Arc::new(Lokinet::default()),
        Arc::new(AmneziaFree::default()),
    ];
    let p = Platform::current();
    v.retain(|e| {
        let inf = e.info();
        inf.platforms.is_empty() || inf.platforms.contains(&p)
    });
    v
}

pub fn by_id(id: &str) -> Option<Arc<dyn Engine>> {
    all().into_iter().find(|e| e.info().id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remaining_road_engines_are_wired() {
        let ids: Vec<String> = all().into_iter().map(|e| e.info().id).collect();
        for id in [
            "psiphon",
            "riseup",
            "lokinet",
            "tor-snowflake",
            "tor-webtunnel",
            "tor-obfs4",
            "amnezia-free",
            "warp",
            "tor",
        ] {
            assert!(ids.iter().any(|x| x == id), "missing {id} in {ids:?}");
        }
    }
}
