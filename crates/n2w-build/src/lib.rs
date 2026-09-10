//! Orchestrate analyze → verify → (planned) WASI cross of the closure.
//!
//! Fail closed. Do not emit a fake `.wasm` that "links". North star:
//! `n2w build nixpkgs#foot` produces `foot.wpm` with a real `foot.wasm`.

use n2w_analyze::{analyze, Analysis, AnalyzeError, PackageStatus};
use n2w_nix::{plan as nix_plan, NixError, NixPlan};
use n2w_package::{planned_tree, WpmManifest};
use n2w_patch::{plan as patch_plan, PatchPlan};
use n2w_verify::{verify_appstore_runtime, VerifyError, VerifyReport};

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error(transparent)]
    Analyze(#[from] AnalyzeError),
    #[error(transparent)]
    Verify(#[from] VerifyError),
    #[error(transparent)]
    Nix(#[from] NixError),
    #[error("{0} is already shipping from Relay / repo.wawona.io; this is not a nixpkgs conversion")]
    AlreadyShipping(String),
    #[error("WASI cross for {name} is planned. Refusing to write a stub wasm.")]
    CrossNotReady { name: String },
}

#[derive(Debug, Clone)]
pub struct BuildPlan {
    pub analysis: Analysis,
    pub nix: Option<NixPlan>,
    pub patch: PatchPlan,
    pub verify: VerifyReport,
    pub manifest: WpmManifest,
    pub artifact_tree: String,
}

pub fn plan_build(name: &str) -> Result<BuildPlan, BuildError> {
    let analysis = analyze(name)?;
    let verify = verify_appstore_runtime(&analysis)?;
    let nix = match nix_plan(&analysis) {
        Ok(plan) => Some(plan),
        Err(NixError::NotNixpkgs(_)) if analysis.spec.package.status == PackageStatus::Shipping => {
            None
        }
        Err(err) => return Err(err.into()),
    };
    let patch = patch_plan(&analysis);
    let manifest = WpmManifest::from_analysis(&analysis);
    let artifact_tree = planned_tree(&analysis.spec.package.name);
    Ok(BuildPlan {
        analysis,
        nix,
        patch,
        verify,
        manifest,
        artifact_tree,
    })
}

/// Run the build. Shipping non-nixpkgs packages are reported, not rebuilt.
/// nixpkgs conversions fail closed until the cross toolchain lands.
pub fn build(name: &str) -> Result<BuildPlan, BuildError> {
    let plan = plan_build(name)?;
    if plan.analysis.spec.package.status == PackageStatus::Shipping && plan.nix.is_none() {
        return Err(BuildError::AlreadyShipping(
            plan.analysis.spec.package.name.clone(),
        ));
    }
    Err(BuildError::CrossNotReady {
        name: plan.analysis.spec.package.name.clone(),
    })
}
