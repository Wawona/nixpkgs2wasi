# Wawona App Store runtime profile

`n2w verify` checks this list. It is **not** App Review.

Required:

- bytecode is WASI P1 or P2 WASM
- Pulley can interpret it (no Cranelift / `MAP_JIT` required on Apple mobile)
- no Mach-O, ELF, or `.deb` payload in the WPM tree
- capabilities declared (Wayland, PTY, filesystem, network, graphics, audio)
- network and audio default off on `profile = "strict"`
- repository URL is `repo.wawona.io` `/wasm` (never APT `/Packages`)
- no private Apple API required by the package
- no host `fork`/`exec` of unsigned native code

Not claimed:

- Apple approved the package
- Apple approved a downloadable application catalog
- the package is a native iOS app

Distribution (bundled vs downloadable) is decided by Wawona / WPM policy,
per platform, without changing this compiler.
