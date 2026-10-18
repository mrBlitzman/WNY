use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Protection,
    Unblock,
    Private,
    Emergency,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Protection => "protection",
            Mode::Unblock => "unblock",
            Mode::Private => "private",
            Mode::Emergency => "emergency",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "protection" | "protect" => Some(Mode::Protection),
            "unblock" => Some(Mode::Unblock),
            "private" => Some(Mode::Private),
            "emergency" => Some(Mode::Emergency),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Slot {
    Names,
    Filter,
    Road,
}

impl Slot {
    pub fn as_str(self) -> &'static str {
        match self {
            Slot::Names => "names",
            Slot::Filter => "filter",
            Slot::Road => "road",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "names" | "name" => Some(Slot::Names),
            "filter" => Some(Slot::Filter),
            "road" => Some(Slot::Road),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum SlotChoice {
    Auto,
    Off,
    Pin { engine: String },
}

impl Default for SlotChoice {
    fn default() -> Self {
        SlotChoice::Auto
    }
}

impl SlotChoice {
    pub fn parse(s: &str) -> Self {
        let t = s.trim();
        if t.eq_ignore_ascii_case("auto") || t.is_empty() {
            SlotChoice::Auto
        } else if t.eq_ignore_ascii_case("off") {
            SlotChoice::Off
        } else {
            SlotChoice::Pin {
                engine: t.to_string(),
            }
        }
    }

    pub fn is_auto(&self) -> bool {
        matches!(self, SlotChoice::Auto)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    En,
    Tr,
    De,
}

impl Default for Language {
    fn default() -> Self {
        Language::En
    }
}

impl Language {
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "tr" | "tr-tr" | "turkish" | "türkçe" => Language::Tr,
            "de" | "de-de" | "german" | "deutsch" => Language::De,
            _ => Language::En,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Tr => "tr",
            Language::De => "de",
        }
    }
}
