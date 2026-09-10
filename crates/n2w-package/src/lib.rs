//! WPM artifact layout for `repo.wawona.io/wasm/v1`.
//!
//! Prefer a package with metadata and assets, not a naked `.wasm`. Store `wpm`
//! fetches `/wasm/v1` only. Never APT, `/Packages`, `/jailbreak/`, or `/termux/`.

use n2w_analyze::Analysis;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WpmManifest {
    pub name: String,
    pub source: String,
    pub entrypoint: String,
    pub wasi: String,
    pub runtime: String,
    pub capabilities: n2w_analyze::Capabilities,
    pub repository: String,
}

impl WpmManifest {
    pub fn from_analysis(analysis: &Analysis) -> Self {
        let wasi = if analysis.spec.wasi.preview >= 2 {
            "preview2".to_string()
        } else {
            "preview1".to_string()
        };
        Self {
            name: analysis.spec.package.name.clone(),
            source: analysis.spec.package.source.clone(),
            entrypoint: format!("bin/{}.wasm", analysis.spec.package.name),
            wasi,
            runtime: "wawona-1".into(),
            capabilities: analysis.spec.capabilities.clone(),
            repository: analysis.spec.wpm.repository.clone(),
        }
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

/// Logical tree a successful build must produce. Not written until cross-compile exists.
pub fn planned_tree(name: &str) -> String {
    format!(
        "{name}.wpm\n\
         ├── manifest.toml\n\
         ├── bin/\n\
         │   └── {name}.wasm\n\
         ├── share/\n\
         └── licenses/\n"
    )
}
