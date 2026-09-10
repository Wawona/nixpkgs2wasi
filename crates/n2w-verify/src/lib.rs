//! Technical App Store *runtime profile* checks.
//!
//! This is not an Apple review oracle. Passing means the package is WASI
//! bytecode for Relay Pulley, with declared capabilities, and no native
//! payload. Whether Wawona may expose that package as a downloadable catalog
//! entry on iOS is a separate WPM distribution-policy question.

use n2w_analyze::{Analysis, PackageStatus};

#[derive(Debug, thiserror::Error)]
pub enum VerifyError {
    #[error("appstore.profile must be \"strict\" (got {0})")]
    Profile(String),
    #[error("WASI preview must be 1 or 2 (got {0})")]
    WasiPreview(u8),
    #[error("wpm.repository must be the wasm channel (got {0})")]
    Repository(String),
    #[error("network capability is not granted on the strict runtime profile")]
    NetworkForbidden,
    #[error("audio capability is not granted on the strict runtime profile yet")]
    AudioForbidden,
    #[error("package status is unsupported")]
    Unsupported,
}

#[derive(Debug, Clone)]
pub struct VerifyReport {
    pub ok: bool,
    pub profile: &'static str,
    pub notes: Vec<String>,
}

pub fn verify_appstore_runtime(analysis: &Analysis) -> Result<VerifyReport, VerifyError> {
    if analysis.spec.package.status == PackageStatus::Unsupported {
        return Err(VerifyError::Unsupported);
    }
    if analysis.spec.appstore.profile != "strict" {
        return Err(VerifyError::Profile(
            analysis.spec.appstore.profile.clone(),
        ));
    }
    if analysis.spec.wasi.preview != 1 && analysis.spec.wasi.preview != 2 {
        return Err(VerifyError::WasiPreview(analysis.spec.wasi.preview));
    }
    let repo = &analysis.spec.wpm.repository;
    if !(repo.contains("repo.wawona.io") && repo.contains("/wasm")) {
        return Err(VerifyError::Repository(repo.clone()));
    }
    if analysis.spec.capabilities.network {
        return Err(VerifyError::NetworkForbidden);
    }
    if analysis.spec.capabilities.audio {
        return Err(VerifyError::AudioForbidden);
    }

    Ok(VerifyReport {
        ok: true,
        profile: "Wawona App Store Runtime Profile",
        notes: vec![
            "WASM only. Pulley-compatible. No JIT required to run.".into(),
            "No native executable payload in the WPM tree.".into(),
            "No private Apple API requirement in the package itself.".into(),
            "Passing this verifier is not App Review approval.".into(),
            "Distribution policy (bundled vs downloadable catalog) is decided outside this crate.".into(),
        ],
    })
}
