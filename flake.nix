{
  description = "nixpkgs2wasi: curated nixpkgs Wayland clients to WASI P1/P2 for Wawona";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs = { self, nixpkgs }:
    let
      # Nixpkgs 26.11 throws on x86_64-darwin eval.
      systems = [ "aarch64-darwin" "x86_64-linux" "aarch64-linux" ];
      forAll = nixpkgs.lib.genAttrs systems;
    in {
      packages = forAll (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          n2w = pkgs.rustPlatform.buildRustPackage {
            pname = "nixpkgs2wasi";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            postInstall = ''
              mkdir -p $out/share/nixpkgs2wasi
              cp -R packages $out/share/nixpkgs2wasi/
            '';
          };
        in {
          default = n2w;
          n2w = n2w;
        });

      apps = forAll (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/n2w";
        };
      });
    };
}
