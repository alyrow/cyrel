{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
      };
      lib = pkgs.lib;
      rust-nightly = fenix.packages.${system};
    in {
      devShell = pkgs.mkShell {
        nativeBuildInputs = lib.singleton (with rust-nightly; combine (with default; [
          cargo
          rustc
          rust-std
          clippy-preview
          rustfmt-preview
          latest.rust-src
        ])) ++ (with pkgs; [
          pkg-config
          openssl
          sqlx-cli
          postgresql
          rust-nightly.rust-analyzer
          cargo-expand
        ]);
      };
    });
}
