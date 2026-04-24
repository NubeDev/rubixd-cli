//! Output formatting — the only module that writes to stdout.
//!
//! Two modes:
//!  * [`Format::Human`] — pretty-printed text aimed at humans.
//!  * [`Format::Json`]  — raw JSON for scripts / AI agents (`--json` flag).

use serde_json::Value;

/// Selects how results are printed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Human-readable, lightly formatted.
    Human,
    /// Raw JSON; consumer is responsible for interpretation.
    Json,
}

impl Format {
    /// Convert the `--json` boolean flag to a [`Format`] variant.
    pub fn from_json_flag(json: bool) -> Self {
        if json { Self::Json } else { Self::Human }
    }

    /// Print `value` according to the selected format.
    /// `human_fn` is called for [`Format::Human`]; it receives the JSON value
    /// so it can extract the fields it needs.
    pub fn print(self, value: &Value, human_fn: impl Fn(&Value)) {
        match self {
            Self::Json => println!("{}", value),
            Self::Human => human_fn(value),
        }
    }
}

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Return the string at `key`, falling back to `"-"` for missing / null.
pub fn str_field<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or("-")
}

/// Return the boolean at `key`, falling back to `false`.
pub fn bool_field(v: &Value, key: &str) -> bool {
    v.get(key).and_then(|x| x.as_bool()).unwrap_or(false)
}
