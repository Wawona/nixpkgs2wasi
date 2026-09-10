//! Build-system adaptation for curated packages.
//!
//! Wawona-owned patch logic is Rust. Upstream C/C++ stays upstream. Do not
//! rewrite the client. Substitute the platform under it.

use n2w_analyze::Analysis;

#[derive(Debug, Clone)]
pub struct PatchPlan {
    pub needed: bool,
    pub notes: Vec<String>,
}

pub fn plan(analysis: &Analysis) -> PatchPlan {
    let mut notes = Vec::new();
    if analysis.spec.wawona.wayland {
        notes.push(
            "Keep libwayland-client. The app still speaks Wayland. Relay provides the socket."
                .into(),
        );
        notes.push(
            "Do not translate Wayland into UIKit/AppKit/Jetpack per application.".into(),
        );
    }
    if analysis
        .spec
        .wawona
        .graphics
        .iter()
        .any(|g| g == "drm" || g == "gbm")
    {
        notes.push(
            "DRM/KMS/GBM ABI is wwn-iland userspace. Never open a real /dev/dri.".into(),
        );
    }
    if analysis.spec.package.native_port == "wwn-foot" {
        notes.push(
            "Native Mach-O/in-process foot stays in wwn-foot. Wasm foot is the catalog path, not a replacement."
                .into(),
        );
    }
    PatchPlan {
        needed: !notes.is_empty(),
        notes,
    }
}
