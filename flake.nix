{
  description = "A minimal flake template that you can adapt to your own project";

  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixos-unstable/nixexprs.tar.zst";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, rust-overlay, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      perSystem =
        {
          config,
          pkgs,
          system,
          ...
        }:
        let
          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          };
        in
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };

          devShells.default = pkgs.mkShell {
            packages =
              with pkgs;
              [
                config.formatter
                toolchain
                bacon
                cargo-nextest
                cargo-dist
              ]
              ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
                mold
                clang
              ];

          };

          packages = rec {
            g-codifier = rustPlatform.buildRustPackage rec {
              pname = cargoToml.package.name;
              version = cargoToml.package.name;
              cargoLock.lockFile = ./Cargo.lock;
              useNextest = true;

              nativeBuildInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
                pkgs.mold
                pkgs.clang
              ];

              src = pkgs.lib.cleanSource ./.;
              meta = {
                description = cargoToml.package.description;
                homepage = "https://github.com/FraI3mega/g-codifier";
                changelog = "https://github.com/FraI3mega/g-codifier/releases/tag/v${version}";
                license = pkgs.lib.licenses.asl20;
                mainProgram = "g-codifier";
              };
            };
            default = g-codifier;
          };

          formatter = pkgs.nixfmt;
        };
    };
}
