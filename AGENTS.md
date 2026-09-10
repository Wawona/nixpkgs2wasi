# nixpkgs2wasi

Rust CLI. Curated nixpkgs Wayland clients → WASI P1/P2 for Wawona Relay.
Hosted at `repo.wawona.io/wasm/v1`. Not a Linux VM. Not QEMU/UTM.

Query **wwn-mcp** before changing architecture. Skill `wawona-nixpkgs2wasi`.
DAG: L3′, nixpkgs-only. Never an input of L0-L3.

## Hard rejects

- Stub `.wasm` that "builds"
- Auto-mirror nixpkgs
- Claim Apple App Review approved a package
- UIKit translator instead of Wayland
- Real `/dev/dri`
- Mix APT debs into `/wasm`

Wawona-owned code is Rust. Upstream apps stay in their upstream language.
No em dash.
