{ pkgs ? import <nixpkgs> {} }:

let
  manifest = (pkgs.lib.importTOML ../cli/Cargo.toml).package;
in
pkgs.rustPlatform.buildRustPackage {
  pname = manifest.name;
  version = manifest.version;

  src = pkgs.lib.cleanSourceWith {
    src = ../.;
    filter = path: type:
      let
        base = baseNameOf path;
      in
        !(type == "directory" && (base == "target" || base == ".git"));
  };

  cargoRoot = "cli";
  buildAndTestSubdir = "cli";

  cargoLock = {
    lockFile = ../cli/Cargo.lock;
  };

  meta = with pkgs.lib; {
    description = manifest.description;
    homepage = "https://github.com/DarkRader/macicon";
    license = licenses.mit;
    platforms = platforms.darwin;
    mainProgram = "macicon";
  };
}
