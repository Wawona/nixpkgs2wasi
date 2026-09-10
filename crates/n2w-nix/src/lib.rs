//! Plan a nixpkgs evaluation as `wasm32-wasip1` / `wasm32-wasip2`.
//!
//! This crate does not eval nixpkgs yet. It records the intended host swap:
//! Linux disappears as a runtime. The application and its userspace closure
//! cross-compile against WASI + Wawona POSIX + Wayland + wwn-iland.

use n2w_analyze::Analysis;

#[derive(Debug, thiserror::Error)]
pub enum NixError {
    #[error("source {0} is not a nixpkgs attribute (expected nixpkgs#name)")]
    NotNixpkgs(String),
}

#[derive(Debug, Clone)]
pub struct NixPlan {
    pub attr: String,
    pub conceptual_platform: &'static str,
    pub llvm_target: &'static str,
    pub note: &'static str,
}

pub fn plan(analysis: &Analysis) -> Result<NixPlan, NixError> {
    let source = &analysis.spec.package.source;
    if !source.starts_with("nixpkgs#") {
        return Err(NixError::NotNixpkgs(source.clone()));
    }
    Ok(NixPlan {
        attr: source.clone(),
        conceptual_platform: "wasm32-wawona",
        llvm_target: analysis.llvm_target,
        note: "Cross the derivation and its userspace closure. Do not put a Linux kernel inside WASM. Do not call QEMU or UTM.",
    })
}
