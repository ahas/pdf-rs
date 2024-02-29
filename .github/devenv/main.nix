# This is your devenv configuration
{ pkgs, ... }:
pkgs.devshell.mkShell (
  { config
  , extraModulesPath
  , ...
  }:
  {
    imports = [
      (extraModulesPath + "/language/c.nix")
      (extraModulesPath + "/language/rust.nix")
    ];

    env = [

    ];

    devshell.packages = with pkgs; [
      cmake
      (pkgs.rust-bin.stable.latest.default.override {
        extensions = [ "rust-src" ];
      })
    ];

    language.c.compiler = pkgs.gcc;
    language.rust.enableDefaultToolchain = false;

    # C/FFI dependencies
    language.c.includes = [
    ];

    commands = [
    ];
  }
)
