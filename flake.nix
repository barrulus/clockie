{
  description = "clockie — lightweight Wayland layer-shell desktop clock";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Native dependencies needed for Wayland + layer-shell
        nativeBuildInputs = with pkgs; [
          pkg-config
        ];

        buildInputs = with pkgs; [
          wayland
          wayland-protocols
          libxkbcommon
        ];

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;
          inherit nativeBuildInputs buildInputs;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        clockie = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });
      in
      {
        checks = {
          inherit clockie;
          clockie-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });
          clockie-fmt = craneLib.cargoFmt { src = commonArgs.src; };
        };

        packages = {
          default = clockie;
          inherit clockie;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = clockie;
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = with pkgs; [
            rustToolchain
            cargo-watch
            cargo-expand
          ] ++ nativeBuildInputs ++ buildInputs;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;

          WAYLAND_PROTOCOLS = "${pkgs.wayland-protocols}/share/wayland-protocols";
        };
      }
    );
}
