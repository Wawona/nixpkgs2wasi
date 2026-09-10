# nixpkgs2wasi

Curated **nixpkgs** Wayland clients, cross-compiled to **WASI Preview 1 / Preview 2**, hosted at [`repo.wawona.io/wasm`](https://repo.wawona.io/search/?channel=wasm), and executed by **Wawona Relay** on Wawona Compositor.

This is a **Rust** project. It is not a Linux distribution, not a kernel, and not QEMU/UTM.

```text
nixpkgs#foot
        │
        ▼
nixpkgs2wasi (this repo)
        │  userspace closure → wasm32-wasip1 / wasm32-wasip2
        ▼
foot.wpm  (bytecode + manifest + assets)
        │
        ▼
repo.wawona.io/wasm/v1
        │
        ▼
wpm install foot
        │
        ▼
Wawona Relay (Pulley on Apple mobile store artifacts)
        │
        ├─ WASI / Wawona POSIX
        ├─ real Wayland → Wawona Compositor (Smithay)
        └─ DRM/KMS/GBM ABI → wwn-iland (userspace)
                │
                ▼
iPhone / iPad / Mac / Android / Linux / Watch / tv / vision
```

North star:

```bash
n2w build nixpkgs#foot
wpm install foot
```

A real Wayland window on Wawona. No Linux VM.

`hello-wasi-gui` already proves the Wayland-in-WASI path (`wl_shm` + `xdg_wm_base`) on every product target, including watchOS.

## What this repo is

| This repo | Not this repo |
|-----------|----------------|
| Curated package list + CLI (`n2w`) | Auto-mirror of all nixpkgs |
| Cross-compile *userspace* to WASI | Linux kernel inside WASM |
| WPM package production | Relay Pulley / Cranelift engine |
| App Store **runtime profile** verifier | Apple App Review |
| Wayland stays Wayland | UIKit/AppKit translator per app |

Execute engines stay in **Relay** (`wwn-relay`). Graphics personality stays in **wwn-iland**. Catalog hosting stays in **repo.wawona.io**. Native Mach-O ports (example: `wwn-foot`) stay first-class. Wasm is the long-tail catalog path.

Layer: **L3′**, nixpkgs-only. Never a flake input of L0-L3. L4 Wawona consumes packages through `/wasm/v1`, not by importing this crate into the toolchain.

## Two questions (do not combine)

1. **Can this Unix/Wayland program become safe WASI bytecode?** (`n2w analyze` / `n2w verify` / later `n2w build`)
2. **May Wawona distribute or install that artifact on this platform?** (WPM policy: bundled catalog vs downloadable catalog)

Guideline 2.5.2 and the Apple Developer Program License Agreement still govern whether a downloadable application catalog is allowed inside the store app. Passing the runtime profile means:

- WASM only
- Pulley-compatible
- no JIT required
- no native executable payload
- declared Wawona capabilities
- no private Apple API requirement
- no host process execution

It does **not** mean "Apple approved this package."

## CLI

```bash
n2w catalog
n2w show foot
n2w analyze nixpkgs#foot
n2w plan foot
n2w verify foot
n2w build nixpkgs#foot
```

`n2w build` fail-closes until the WASI closure actually compiles. It will not write a stub `.wasm`.

`hello-wasi-gui` is already shipping on `/wasm/v1`. `n2w build hello-wasi-gui` points at `wpm install`.

## Workspace

```text
nixpkgs2wasi/
├── crates/nixpkgs2wasi   CLI (`n2w`)
├── crates/n2w-nix        nixpkgs evaluation plan
├── crates/n2w-build      orchestration (fail closed)
├── crates/n2w-analyze    catalog + tiers
├── crates/n2w-patch      build-system adaptation notes
├── crates/n2w-package    WPM manifest
├── crates/n2w-verify     runtime profile
├── packages/             curated allow-list
├── wit/                  host ABI the bytecode may import
└── nix/                  later: pkgsCross.wawona overlay
```

Conceptual Nix platform: `wasm32-wawona`. LLVM targets remain `wasm32-wasip1` / `wasm32-wasip2`.

## Compatibility tiers

| Tier | Meaning |
|------|---------|
| WASI | Standard WASI only |
| Wawona POSIX | Relay Unix compatibility (PTY, poll, files) |
| Wawona Wayland | GUI via Wawona Compositor |
| Wawona Graphics | wwn-iland / accelerated ABI |
| Unsupported | Do not convert |

Progression: `hello` → `jq` → `foot` → GTK → Mesa. Do not start at Firefox.

## Hard rejects

- Auto-publishing every successful nixpkgs build
- Claiming App Store / App Review approval
- Linux kernel, QEMU, UTM, or virtio-gpu inside the package
- A Mode B flavor of the Runtime catalog (bytecode stays `/wasm/v1`; Mode B may JIT-execute the same files)
- Mixing `.deb` into this catalog
- Re-hosting a Wayland client onto a fake KMS-only path because Wayland-EGL is unfinished
- Writing the compatibility platform in C because the upstream app is C
- Making this repo a flake input of `wwn-toolchain` or `wwn-iland`

## License

MIT for Wawona tooling. Upstream package licenses travel with each WPM artifact.
