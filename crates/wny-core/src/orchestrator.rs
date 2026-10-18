use crate::engine::{Engine, Health};
use crate::engines;
use crate::error::Result;
use crate::i18n;
use crate::mode::{Language, Mode, Slot, SlotChoice};
use crate::platform::Platform;
use crate::policy::Policy;
use crate::probe::{LiveProbes, ProbeReport, Probes};
use crate::settings::{self, Settings};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotView {
    pub slot: String,
    pub choice: String,
    pub engine: Option<String>,
    pub engine_label: String,
    pub health: String,
    pub available: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSnapshot {
    pub on: bool,
    pub mode: String,
    pub language: String,
    pub wizard_done: bool,
    pub start_at_login: bool,
    pub status_key: String,
    pub status: String,
    pub names: SlotView,
    pub filter: SlotView,
    pub road: SlotView,
    pub pin_failed: Option<String>,
    pub platform: String,
    pub has_filter: bool,
    pub catalog: serde_json::Value,
}

struct Running {
    id: String,
    engine: Arc<dyn Engine>,
}

pub struct Orchestrator {
    home: PathBuf,
    settings: Settings,
    policy: Policy,
    engines: BTreeMap<String, Arc<dyn Engine>>,
    probes: Box<dyn Probes>,
    running: BTreeMap<Slot, Running>,
    last_status: String,
    pin_failed: Option<String>,
    captive_until: Option<Instant>,
}

impl Orchestrator {
    pub fn open_default() -> Result<Self> {
        Self::open(settings::data_dir()?, Box::new(LiveProbes), engines::all())
    }

    pub fn open(
        home: PathBuf,
        probes: Box<dyn Probes>,
        engine_list: Vec<Arc<dyn Engine>>,
    ) -> Result<Self> {
        std::fs::create_dir_all(&home)?;
        std::env::set_var("WNY_HOME", home.as_os_str());
        let settings = settings::load_from(&home)?;
        let mut engines = BTreeMap::new();
        for e in engine_list {
            engines.insert(e.info().id.clone(), e);
        }
        Ok(Self {
            home,
            settings,
            policy: Policy::bundled()?,
            engines,
            probes,
            running: BTreeMap::new(),
            last_status: "idle".into(),
            pin_failed: None,
            captive_until: None,
        })
    }

    pub fn snapshot(&self) -> UiSnapshot {
        let lang = self.settings.language;
        UiSnapshot {
            on: self.settings.on,
            mode: self.settings.mode.as_str().into(),
            language: lang.as_str().into(),
            wizard_done: self.settings.wizard_done,
            start_at_login: self.settings.start_at_login,
            status_key: self.last_status.clone(),
            status: i18n::lookup(lang, &format!("status.{}", self.last_status)),
            names: self.slot_view(Slot::Names, lang),
            filter: self.slot_view(Slot::Filter, lang),
            road: self.slot_view(Slot::Road, lang),
            pin_failed: self.pin_failed.clone(),
            platform: Platform::current().as_str().into(),
            has_filter: Platform::current().has_filter_slot(),
            catalog: i18n::catalog(lang),
        }
    }

    fn slot_view(&self, slot: Slot, lang: Language) -> SlotView {
        let choice = self.settings.choice(slot);
        let choice_s = match choice {
            SlotChoice::Auto => "auto",
            SlotChoice::Off => "off",
            SlotChoice::Pin { .. } => "pin",
        };
        let engine = self.running.get(&slot).map(|r| r.id.clone());
        let engine_label = engine
            .as_deref()
            .map(|id| i18n::engine_label(lang, id))
            .unwrap_or_else(|| i18n::lookup(lang, "choice.off"));
        let health = self
            .running
            .get(&slot)
            .map(|r| format!("{:?}", r.engine.health()).to_lowercase())
            .unwrap_or_else(|| "off".into());
        let available = self
            .policy
            .order(self.settings.mode, slot)
            .unwrap_or(&[])
            .iter()
            .filter(|id| self.engines.contains_key(id.as_str()))
            .cloned()
            .collect();
        SlotView {
            slot: slot.as_str().into(),
            choice: choice_s.into(),
            engine,
            engine_label,
            health,
            available,
        }
    }

    pub fn set_language(&mut self, lang: Language) -> Result<UiSnapshot> {
        self.settings.language = lang;
        self.persist()?;
        Ok(self.snapshot())
    }

    pub fn finish_wizard(&mut self, start_at_login: bool) -> Result<UiSnapshot> {
        self.settings.wizard_done = true;
        self.settings.start_at_login = start_at_login;
        self.persist()?;
        Ok(self.snapshot())
    }

    pub fn set_mode(&mut self, mode: Mode) -> Result<UiSnapshot> {
        self.settings.mode = mode;
        self.persist()?;
        if self.settings.on {
            self.recompute()?;
        }
        Ok(self.snapshot())
    }

    pub fn set_slot(&mut self, slot: Slot, choice: SlotChoice) -> Result<UiSnapshot> {
        self.settings.set_choice(slot, choice);
        self.persist()?;
        if self.settings.on {
            self.recompute()?;
        }
        Ok(self.snapshot())
    }

    pub fn set_on(&mut self, on: bool) -> Result<UiSnapshot> {
        self.settings.on = on;
        if on {
            self.recompute()?;
        } else {
            self.stop_all();
            self.last_status = "idle".into();
            self.pin_failed = None;
        }
        self.persist()?;
        Ok(self.snapshot())
    }

    pub fn new_identity(&mut self) -> Result<UiSnapshot> {
        if let Some(r) = self.running.get(&Slot::Road) {
            r.engine.new_identity()?;
        }
        Ok(self.snapshot())
    }

    pub fn open_captive_portal(&mut self) -> Result<UiSnapshot> {
        let secs = self.policy.captive_portal_secs;
        self.captive_until = Some(Instant::now() + Duration::from_secs(secs));
        self.last_status = "captive".into();
        Ok(self.snapshot())
    }

    pub fn captive_active(&self) -> bool {
        self.captive_until
            .map(|t| Instant::now() < t)
            .unwrap_or(false)
    }

    pub fn probes(&self) -> ProbeReport {
        self.probes.run(self.settings.mode, &self.policy)
    }

    fn persist(&self) -> Result<()> {
        settings::save_to(&self.home, &self.settings)
    }

    fn stop_all(&mut self) {
        let running = std::mem::take(&mut self.running);
        for (_, r) in running {
            let _ = r.engine.stop();
        }
    }

    fn stop_slot(&mut self, slot: Slot) {
        if let Some(r) = self.running.remove(&slot) {
            let _ = r.engine.stop();
        }
    }

    fn recompute(&mut self) -> Result<()> {
        self.pin_failed = None;
        let report = self.probes.run(self.settings.mode, &self.policy);
        let mode = self.settings.mode;

        self.apply_slot(Slot::Names, mode, &report)?;
        if Platform::current().has_filter_slot() {
            self.apply_slot(Slot::Filter, mode, &report)?;
        } else {
            self.stop_slot(Slot::Filter);
        }
        self.apply_slot(Slot::Road, mode, &report)?;

        self.last_status = if self.pin_failed.is_some() {
            "pinFailed".into()
        } else if report.tls_mitm {
            "mitm".into()
        } else if mode == Mode::Private {
            if self.running.get(&Slot::Road).is_none() {
                "failClosed".into()
            } else {
                "private".into()
            }
        } else if mode == Mode::Unblock || mode == Mode::Emergency {
            if !report.connectivity_ok && self.running.get(&Slot::Road).is_none() {
                "unblocking".into()
            } else if !report.connectivity_ok {
                "unblocking".into()
            } else {
                "protecting".into()
            }
        } else if self.running.is_empty() && mode == Mode::Protection {
            "protecting".into()
        } else {
            "protecting".into()
        };

        if (mode == Mode::Private || mode == Mode::Emergency)
            && self.policy.fail_closed(mode)
            && self.running.get(&Slot::Road).is_none()
            && !matches!(self.settings.road, SlotChoice::Off)
        {
            self.last_status = "failClosed".into();
        }
        Ok(())
    }

    fn apply_slot(&mut self, slot: Slot, mode: Mode, report: &ProbeReport) -> Result<()> {
        let choice = self.settings.choice(slot).clone();
        match choice {
            SlotChoice::Off => {
                if slot == Slot::Road && report.tls_mitm && mode == Mode::Protection {
                    // MITM on Protection forces WARP even if Road is Off in the template.
                    let order = self
                        .policy
                        .mode(mode)
                        .ok()
                        .map(|m| m.road.on_mitm.clone())
                        .unwrap_or_default();
                    self.try_order(slot, &order, true)?;
                } else {
                    self.stop_slot(slot);
                }
            }
            SlotChoice::Pin { engine } => {
                self.stop_slot(slot);
                match self.start_id(slot, &engine) {
                    Ok(()) => {}
                    Err(_) => {
                        self.pin_failed = Some(engine);
                    }
                }
            }
            SlotChoice::Auto => {
                if slot == Slot::Filter {
                    let mp = self.policy.mode(mode)?;
                    if mp.filter.only_if_road_bootstrap_fails {
                        // Start filter only when overlay cannot bootstrap. First
                        // attempt road without filter; orchestrator calls Filter
                        // after Road, so we inspect previous road health on next
                        // tick. For a single pass: skip Filter unless probes show
                        // no connectivity and road is down.
                        if report.connectivity_ok {
                            self.stop_slot(slot);
                            return Ok(());
                        }
                    }
                    if mode == Mode::Protection {
                        self.stop_slot(slot);
                        return Ok(());
                    }
                }
                if slot == Slot::Road && mode == Mode::Protection && !report.tls_mitm {
                    self.stop_slot(slot);
                    return Ok(());
                }
                let order = self.policy.order(mode, slot)?.to_vec();
                self.stop_slot(slot);
                self.try_order(slot, &order, false)?;
            }
        }
        Ok(())
    }

    fn try_order(&mut self, slot: Slot, order: &[String], pin_like: bool) -> Result<()> {
        for id in order {
            if self.start_id(slot, id).is_ok() {
                return Ok(());
            }
        }
        if pin_like {
            self.pin_failed = order.first().cloned();
        }
        Ok(())
    }

    fn start_id(&mut self, slot: Slot, id: &str) -> Result<()> {
        let Some(engine) = self.engines.get(id).cloned() else {
            return Err(crate::error::Error::Engine {
                id: id.into(),
                detail: "unknown".into(),
            });
        };
        if engine.info().slot != slot {
            return Err(crate::error::Error::Engine {
                id: id.into(),
                detail: "wrong slot".into(),
            });
        }
        match engine.health() {
            Health::Unsupported => {
                return Err(crate::error::Error::Engine {
                    id: id.into(),
                    detail: "unsupported".into(),
                })
            }
            Health::Missing => {
                // Still try start(); process engines return Missing until started.
            }
            _ => {}
        }
        engine.start()?;
        // If start succeeded but health is Missing, treat as failure (no binary).
        match engine.health() {
            Health::Up | Health::Down => {
                // Down immediately after start can mean the process died; still
                // accept Up. For fake engines, start() sets Up.
                if engine.health() == Health::Up {
                    self.running.insert(
                        slot,
                        Running {
                            id: id.into(),
                            engine,
                        },
                    );
                    return Ok(());
                }
                let _ = engine.stop();
                Err(crate::error::Error::Engine {
                    id: id.into(),
                    detail: "did not come up".into(),
                })
            }
            other => {
                let _ = engine.stop();
                Err(crate::error::Error::Engine {
                    id: id.into(),
                    detail: format!("{other:?}"),
                })
            }
        }
    }
}

pub fn shared() -> &'static Mutex<Option<Orchestrator>> {
    static ONCE: Mutex<Option<Orchestrator>> = Mutex::new(None);
    &ONCE
}

pub fn global() -> Result<parking_lot::MutexGuard<'static, Option<Orchestrator>>> {
    let mut g = shared().lock();
    if g.is_none() {
        *g = Some(Orchestrator::open_default()?);
    }
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::EngineInfo;
    use crate::error::Error;
    use crate::probe::FakeProbes;

    struct Fake {
        id: String,
        slot: Slot,
        fail: bool,
        state: Mutex<Health>,
    }

    impl Fake {
        fn ok(id: &str, slot: Slot) -> Arc<dyn Engine> {
            Arc::new(Self {
                id: id.into(),
                slot,
                fail: false,
                state: Mutex::new(Health::Down),
            })
        }
        fn missing(id: &str, slot: Slot) -> Arc<dyn Engine> {
            Arc::new(Self {
                id: id.into(),
                slot,
                fail: true,
                state: Mutex::new(Health::Missing),
            })
        }
    }

    impl Engine for Fake {
        fn info(&self) -> EngineInfo {
            EngineInfo {
                id: self.id.clone(),
                slot: self.slot,
                platforms: vec![],
            }
        }
        fn start(&self) -> Result<()> {
            if self.fail {
                return Err(Error::msg("nope"));
            }
            *self.state.lock() = Health::Up;
            Ok(())
        }
        fn stop(&self) -> Result<()> {
            *self.state.lock() = Health::Down;
            Ok(())
        }
        fn health(&self) -> Health {
            *self.state.lock()
        }
    }

    fn scratch() -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "wny-orch-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn setup(probes: ProbeReport, extras: Vec<Arc<dyn Engine>>) -> Orchestrator {
        let path = scratch();
        std::env::set_var("WNY_DEV", "1");
        let mut list = extras;
        // names fallbacks
        list.push(Fake::missing("quad9", Slot::Names));
        list.push(Fake::ok("cloudflare", Slot::Names));
        list.push(Fake::ok("dns-sb", Slot::Names));
        list.push(Fake::ok("tor-dns", Slot::Names));
        list.push(Fake::ok("goodbyedpi", Slot::Filter));
        list.push(Fake::ok("zapret", Slot::Filter));
        list.push(Fake::ok("spoofdpi", Slot::Filter));
        list.push(Fake::ok("byedpi", Slot::Filter));
        list.push(Fake::missing("warp", Slot::Road));
        list.push(Fake::ok("riseup", Slot::Road));
        list.push(Fake::ok("psiphon", Slot::Road));
        list.push(Fake::ok("tor", Slot::Road));
        list.push(Fake::ok("tor-snowflake", Slot::Road));
        list.push(Fake::ok("tor-webtunnel", Slot::Road));
        list.push(Fake::ok("tor-obfs4", Slot::Road));
        list.push(Fake::ok("lokinet", Slot::Road));
        Orchestrator::open(
            path,
            Box::new(FakeProbes { report: probes }),
            list,
        )
        .unwrap()
    }

    #[test]
    fn auto_names_skips_missing_quad9() {
        let mut o = setup(ProbeReport::default(), vec![]);
        o.set_mode(Mode::Unblock).unwrap();
        let snap = o.set_on(true).unwrap();
        assert_eq!(snap.names.engine.as_deref(), Some("cloudflare"));
        assert_eq!(snap.names.choice, "auto");
    }

    #[test]
    fn pin_does_not_fallback() {
        let mut o = setup(ProbeReport::default(), vec![]);
        o.set_mode(Mode::Unblock).unwrap();
        o.set_slot(
            Slot::Names,
            SlotChoice::Pin {
                engine: "quad9".into(),
            },
        )
        .unwrap();
        let snap = o.set_on(true).unwrap();
        assert!(snap.names.engine.is_none());
        assert_eq!(snap.pin_failed.as_deref(), Some("quad9"));
        assert_eq!(snap.status_key, "pinFailed");
    }

    #[test]
    fn private_uses_tor_and_hidden_dns() {
        let mut o = setup(
            ProbeReport {
                connectivity_ok: true,
                ..ProbeReport::default()
            },
            vec![],
        );
        o.set_mode(Mode::Private).unwrap();
        let snap = o.set_on(true).unwrap();
        assert_eq!(snap.road.engine.as_deref(), Some("tor"));
        assert_eq!(snap.names.engine.as_deref(), Some("tor-dns"));
        assert_eq!(snap.status_key, "private");
    }

    #[test]
    fn private_fail_closed_without_overlay() {
        let path = scratch();
        std::env::set_var("WNY_DEV", "1");
        let list: Vec<Arc<dyn Engine>> = vec![
            Fake::ok("tor-dns", Slot::Names),
            Fake::missing("tor", Slot::Road),
            Fake::missing("tor-snowflake", Slot::Road),
            Fake::missing("tor-webtunnel", Slot::Road),
            Fake::missing("tor-obfs4", Slot::Road),
            Fake::missing("lokinet", Slot::Road),
        ];
        let mut o = Orchestrator::open(
            path,
            Box::new(FakeProbes {
                report: ProbeReport::default(),
            }),
            list,
        )
        .unwrap();
        o.set_mode(Mode::Private).unwrap();
        let snap = o.set_on(true).unwrap();
        assert!(snap.road.engine.is_none());
        assert_eq!(snap.status_key, "failClosed");
    }

    #[test]
    fn settings_only_on_disk() {
        let mut o = setup(ProbeReport::default(), vec![]);
        o.set_language(Language::De).unwrap();
        o.set_on(true).unwrap();
        let entries: Vec<_> = std::fs::read_dir(&o.home)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert!(entries.iter().any(|n| n == "settings.json"));
        assert!(!entries.iter().any(|n| n.contains("telemetry")));
    }

    #[test]
    fn unblock_falls_to_riseup_if_warp_missing() {
        let mut o = setup(
            ProbeReport {
                connectivity_ok: false,
                ..ProbeReport::default()
            },
            vec![],
        );
        o.set_mode(Mode::Unblock).unwrap();
        let snap = o.set_on(true).unwrap();
        assert_eq!(snap.road.engine.as_deref(), Some("riseup"));
    }
}
