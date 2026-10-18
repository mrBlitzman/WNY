use crate::mode::Language;
use serde_json::Value;

const EN: &str = include_str!("../../../i18n/en.json");
const TR: &str = include_str!("../../../i18n/tr.json");
const DE: &str = include_str!("../../../i18n/de.json");

pub fn catalog(lang: Language) -> Value {
    let raw = match lang {
        Language::En => EN,
        Language::Tr => TR,
        Language::De => DE,
    };
    serde_json::from_str(raw).unwrap_or_else(|_| serde_json::from_str(EN).unwrap())
}

pub fn lookup(lang: Language, path: &str) -> String {
    let mut v = catalog(lang);
    for part in path.split('.') {
        v = v.get(part).cloned().unwrap_or(Value::Null);
    }
    match v {
        Value::String(s) => s,
        _ => path.to_string(),
    }
}

pub fn engine_label(lang: Language, id: &str) -> String {
    lookup(lang, &format!("engines.{id}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tr_and_de_have_switch() {
        assert_eq!(lookup(Language::En, "on"), "On");
        assert_eq!(lookup(Language::Tr, "on"), "Açık");
        assert_eq!(lookup(Language::De, "on"), "Ein");
        assert_eq!(lookup(Language::Tr, "slots.road"), "Yol");
    }
}
