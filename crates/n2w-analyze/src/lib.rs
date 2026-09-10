//! Curated catalog loader and compatibility classification.
//!
//! nixpkgs2wasi does not auto-mirror nixpkgs. Every published package is an
//! explicit `packages/<name>/package.toml`.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum AnalyzeError {
    #[error("catalog directory not found (set N2W_CATALOG or run from the repo root)")]
    CatalogMissing,
    #[error("unknown curated package {0}; add packages/{0}/package.toml first")]
    UnknownPackage(String),
    #[error("failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid package.toml at {path}: {source}")]
    Toml {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PackageStatus {
    #[default]
    Planned,
    Shipping,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompatibilityTier {
    /// Standard WASI only.
    Wasi,
    /// Needs Wawona Unix compatibility in Relay.
    WawonaPosix,
    /// GUI app speaking Wayland to Wawona Compositor.
    WawonaWayland,
    /// Uses wwn-iland / accelerated graphics ABI.
    WawonaGraphics,
    Unsupported,
}

impl CompatibilityTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Wasi => "WASI",
            Self::WawonaPosix => "Wawona POSIX",
            Self::WawonaWayland => "Wawona Wayland",
            Self::WawonaGraphics => "Wawona Graphics",
            Self::Unsupported => "Unsupported",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageFile {
    pub package: PackageMeta,
    pub wasi: WasiSpec,
    pub wawona: WawonaSpec,
    pub appstore: AppStoreSpec,
    pub wpm: WpmSpec,
    #[serde(default)]
    pub capabilities: Capabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMeta {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub native_port: String,
    #[serde(default)]
    pub status: PackageStatus,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub license: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasiSpec {
    pub preview: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WawonaSpec {
    #[serde(default)]
    pub wayland: bool,
    #[serde(default)]
    pub posix: Vec<String>,
    #[serde(default)]
    pub graphics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStoreSpec {
    pub profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WpmSpec {
    pub repository: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Capabilities {
    #[serde(default)]
    pub filesystem: bool,
    #[serde(default)]
    pub network: bool,
    #[serde(default)]
    pub pty: bool,
    #[serde(default)]
    pub wayland: bool,
    #[serde(default)]
    pub graphics: bool,
    #[serde(default)]
    pub audio: bool,
}

#[derive(Debug, Clone)]
pub struct Analysis {
    pub spec: PackageFile,
    pub tier: CompatibilityTier,
    pub llvm_target: &'static str,
}

impl Analysis {
    pub fn from_spec(spec: PackageFile) -> Self {
        let tier = classify(&spec);
        let llvm_target = match spec.wasi.preview {
            2 => "wasm32-wasip2",
            _ => "wasm32-wasip1",
        };
        Self {
            spec,
            tier,
            llvm_target,
        }
    }
}

pub fn classify(spec: &PackageFile) -> CompatibilityTier {
    if spec.package.status == PackageStatus::Unsupported {
        return CompatibilityTier::Unsupported;
    }
    let wants_iland = spec.wawona.graphics.iter().any(|g| {
        matches!(g.as_str(), "drm" | "gbm" | "gles" | "vulkan" | "egl")
    });
    if wants_iland {
        return CompatibilityTier::WawonaGraphics;
    }
    if spec.wawona.wayland || spec.capabilities.wayland {
        return CompatibilityTier::WawonaWayland;
    }
    if !spec.wawona.posix.is_empty() || spec.capabilities.pty || spec.capabilities.filesystem {
        return CompatibilityTier::WawonaPosix;
    }
    CompatibilityTier::Wasi
}

pub fn catalog_dir() -> Result<PathBuf, AnalyzeError> {
    if let Ok(p) = std::env::var("N2W_CATALOG") {
        let path = PathBuf::from(p);
        if path.is_dir() {
            return Ok(path);
        }
        return Err(AnalyzeError::CatalogMissing);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(prefix) = exe.parent().and_then(|p| p.parent()) {
            let share = prefix.join("share/nixpkgs2wasi/packages");
            if share.is_dir() {
                return Ok(share);
            }
        }
    }
    let cwd = std::env::current_dir().map_err(|source| AnalyzeError::Io {
        path: PathBuf::from("."),
        source,
    })?;
    let mut cur = cwd.as_path();
    loop {
        let candidate = cur.join("packages");
        if candidate.is_dir() {
            return Ok(candidate);
        }
        match cur.parent() {
            Some(parent) => cur = parent,
            None => break,
        }
    }
    Err(AnalyzeError::CatalogMissing)
}

pub fn list_packages(catalog: &Path) -> Result<Vec<String>, AnalyzeError> {
    let mut names = Vec::new();
    let entries = fs::read_dir(catalog).map_err(|source| AnalyzeError::Io {
        path: catalog.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| AnalyzeError::Io {
            path: catalog.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.join("package.toml").is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

pub fn load_package(catalog: &Path, name: &str) -> Result<PackageFile, AnalyzeError> {
    let name = normalize_package_name(name);
    let path = catalog.join(&name).join("package.toml");
    if !path.is_file() {
        return Err(AnalyzeError::UnknownPackage(name));
    }
    let text = fs::read_to_string(&path).map_err(|source| AnalyzeError::Io {
        path: path.clone(),
        source,
    })?;
    toml::from_str(&text).map_err(|source| AnalyzeError::Toml { path, source })
}

/// Accept `foot`, `nixpkgs#foot`, or `n2w build nixpkgs#foot`.
pub fn normalize_package_name(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some((_, rest)) = trimmed.split_once('#') {
        rest.trim().to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn analyze(name: &str) -> Result<Analysis, AnalyzeError> {
    let catalog = catalog_dir()?;
    let spec = load_package(&catalog, name)?;
    Ok(Analysis::from_spec(spec))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_packages() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages")
    }

    #[test]
    fn foot_is_curated_wayland_north_star() {
        let spec = load_package(&repo_packages(), "nixpkgs#foot").unwrap();
        assert_eq!(spec.package.name, "foot");
        assert_eq!(spec.package.source, "nixpkgs#foot");
        assert_eq!(spec.package.native_port, "wwn-foot");
        let analysis = Analysis::from_spec(spec);
        assert_eq!(analysis.tier, CompatibilityTier::WawonaWayland);
        assert_eq!(analysis.llvm_target, "wasm32-wasip2");
    }

    #[test]
    fn hello_gui_is_already_shipping_p1() {
        let spec = load_package(&repo_packages(), "hello-wasi-gui").unwrap();
        assert_eq!(spec.package.status, PackageStatus::Shipping);
        let analysis = Analysis::from_spec(spec);
        assert_eq!(analysis.tier, CompatibilityTier::WawonaWayland);
        assert_eq!(analysis.llvm_target, "wasm32-wasip1");
    }
}
