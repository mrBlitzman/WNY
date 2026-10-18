use crate::error::{Error, Result};
use crate::mode::{Language, Mode, Slot, SlotChoice};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub language: Language,
    pub mode: Mode,
    pub on: bool,
    pub wizard_done: bool,
    pub start_at_login: bool,
    pub names: SlotChoice,
    pub filter: SlotChoice,
    pub road: SlotChoice,
    pub debug_log: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: Language::En,
            mode: Mode::Protection,
            on: false,
            wizard_done: false,
            start_at_login: false,
            names: SlotChoice::Auto,
            filter: SlotChoice::Auto,
            road: SlotChoice::Auto,
            debug_log: false,
        }
    }
}

impl Settings {
    pub fn choice(&self, slot: Slot) -> &SlotChoice {
        match slot {
            Slot::Names => &self.names,
            Slot::Filter => &self.filter,
            Slot::Road => &self.road,
        }
    }

    pub fn set_choice(&mut self, slot: Slot, choice: SlotChoice) {
        match slot {
            Slot::Names => self.names = choice,
            Slot::Filter => self.filter = choice,
            Slot::Road => self.road = choice,
        }
    }
}

pub fn data_dir() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("WNY_HOME") {
        return Ok(PathBuf::from(p));
    }
    let dirs = ProjectDirs::from("org", "wny", "WNY")
        .ok_or_else(|| Error::Settings("cannot resolve app data dir".into()))?;
    Ok(dirs.data_dir().to_path_buf())
}

pub fn settings_path(dir: &Path) -> PathBuf {
    dir.join("settings.json")
}

pub fn load_from(dir: &Path) -> Result<Settings> {
    let p = settings_path(dir);
    if !p.exists() {
        return Ok(Settings::default());
    }
    let raw = fs::read(&p)?;
    let s: Settings = serde_json::from_slice(&raw)?;
    Ok(s)
}

pub fn save_to(dir: &Path, settings: &Settings) -> Result<()> {
    fs::create_dir_all(dir)?;
    let p = settings_path(dir);
    let tmp = dir.join("settings.json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(settings)?)?;
    fs::rename(&tmp, p)?;
    Ok(())
}

pub fn load() -> Result<Settings> {
    load_from(&data_dir()?)
}

pub fn save(settings: &Settings) -> Result<()> {
    save_to(&data_dir()?, settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch() -> PathBuf {
        let p = std::env::temp_dir().join(format!("wny-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn roundtrip_local_only() {
        let d = scratch();
        let mut s = Settings::default();
        s.language = Language::Tr;
        s.mode = Mode::Private;
        s.road = SlotChoice::Pin {
            engine: "tor".into(),
        };
        save_to(&d, &s).unwrap();
        let loaded = load_from(&d).unwrap();
        assert_eq!(loaded.language, Language::Tr);
        assert_eq!(loaded.mode, Mode::Private);
        assert!(matches!(loaded.road, SlotChoice::Pin { ref engine } if engine == "tor"));
        // nothing else on disk besides settings.json
        let names: Vec<_> = fs::read_dir(&d)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(names.len(), 1);
    }
}
