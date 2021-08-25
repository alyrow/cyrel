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
      rust-nightly = fenix.packages.${system};
    in {
      devShell = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          (with rust-nightly; combine (with default; [
            cargo
            rustc
            rust-std
            rustfmt
            latest.rust-src
          ]))
          rust-nightly.rust-analyzer
          cargo-expand
          sqlite
          (diesel-cli.override {
            sqliteSupport = true;
            postgresqlSupport = false;
            mysqlSupport = false;
          })
        ];
      };
    });
}
