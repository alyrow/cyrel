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
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix, naersk }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
      };
      lib = pkgs.lib;
      rust-nightly = fenix.packages.${system};
      naersk-lib = let
        toolchain = with rust-nightly; combine (with minimal; [
          cargo rustc
        ]);
      in naersk.lib.${system}.override {
        cargo = toolchain;
        rustc = toolchain;
      };

      nativeBuildInputs = with pkgs; [
        pkg-config
      ];

      buildInputs = with pkgs; [
        openssl
      ];
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
          sqlx-cli
          postgresql
          rust-nightly.rust-analyzer
          cargo-expand
        ]) ++ nativeBuildInputs ++ buildInputs;
      };

      defaultPackage = naersk-lib.buildPackage {
        src = ./.;
        inherit nativeBuildInputs buildInputs;
        doCheck = true;
      };
    });
}
