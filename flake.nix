{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-23.11";
    flake-utils.url = "github:numtide/flake-utils";

    # The local dev environment.
    devshell.url = "github:numtide/devshell";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay, devshell, flake-utils }:
    (flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ]
      (system:
        let
          pkgs = (import nixpkgs) {
            inherit system;
            overlays = [
              devshell.overlays.default
              (import rust-overlay)
            ];
          };
        in
        {

          # The dev shell
          devShells.default = pkgs.callPackage ./.github/devenv/main.nix { inherit self; };
        }));
}


