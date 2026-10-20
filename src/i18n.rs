//! Minimale Zweisprachigkeit (Deutsch/Englisch) für Meldungen an die Oberfläche.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    #[default]
    De,
    En,
}

impl Lang {
    pub fn parse(s: &str) -> Self {
        if s.trim().to_ascii_lowercase().starts_with("en") {
            Self::En
        } else {
            Self::De
        }
    }
}

/// `tr!(lang, "deutsch {}", "english {}", args…)` – beide Texte müssen dieselben Platzhalter haben.
macro_rules! tr {
    ($lang:expr, $de:literal, $en:literal $(, $arg:expr)* $(,)?) => {
        match $lang {
            $crate::i18n::Lang::De => format!($de $(, $arg)*),
            $crate::i18n::Lang::En => format!($en $(, $arg)*),
        }
    };
}
