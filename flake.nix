{
  description = "Rust Development Environment for aepc (Arknights: Endfield Pipeline Calculator)";

  inputs = {
    flake-compat = {
      url = "https://git.lix.systems/lix-project/flake-compat/archive/main.tar.gz";
    };

    flake-parts = {
      url = "github:hercules-ci/flake-parts/main";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    nixpkgs = {
      url = "github:NixOS/nixpkgs/nixos-unstable";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay/master";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { flake-parts, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      perSystem =
        { system, pkgs, ... }:
        {
          _module.args.pkgs = (import inputs.nixpkgs) {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };

          devShells.default = pkgs.mkShellNoCC {
            packages = [
              (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
              pkgs.nixfmt
              pkgs.nixfmt-tree
            ];
          };
        };
    };
}
