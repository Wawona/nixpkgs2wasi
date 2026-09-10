# Architecture

## Product split

```text
nixpkgs2wasi     compile curated userspace to WASI
Relay            execute (Pulley / Cranelift). POSIX + Wayland host imports.
wwn-iland        userspace DRM/KMS/GBM personality
Wawona           Smithay compositor + native UI (UniFFI / WWNCore*)
repo.wawona.io   host `/wasm/v1` (store) vs APT debs (not this catalog)
wpm              client installer inside Wawona
```

Linux is not a runtime. The application's userspace closure is retargeted.

Upstream expectation:

```text
foot → libc/POSIX → Wayland → DRM/GBM → Linux
```

Wawona target:

```text
foot.wasm → WASI P1/P2 + Wawona POSIX → Wayland → wwn-iland → Wawona Compositor
```

The app still believes `WAYLAND_DISPLAY` exists. Relay implements the socket.

## DAG

L3′. Flake inputs: nixpkgs only. Do not depend on Wawona (L4). Do not become
an input of `wwn-toolchain` or `wwn-iland`. Optional later merge of the
toolchain fragment is L0 → L3′ (downward), never the reverse.

Native `wwn-foot` remains the in-process Mach-O/static port. Wasm `foot` is
the catalog path.

## Runtime profile vs distribution policy

`n2w verify` answers the compiler/runtime question.

WPM repository policy answers whether a given platform may *download* the
artifact. Store IPA may keep a bundled/allow-listed catalog even when a
public downloadable storefront is refused.

Same `.wpm` bytes. Two exposure modes.

## Host ABI

WIT in `wit/` documents imports Relay already owns. Do not reimplement
Pulley here. Do not add a second wasm product.
