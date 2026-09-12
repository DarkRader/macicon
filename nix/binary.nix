{ pkgs ? import <nixpkgs> {} }:

let
  manifest = (pkgs.lib.importTOML ../cli/Cargo.toml).package;
  version = manifest.version;
in
pkgs.stdenv.mkDerivation {
  pname = manifest.name;
  inherit version;

  src = pkgs.fetchurl {
    url = "https://github.com/DarkRader/macicon/releases/download/v${version}/macicon-v${version}-macos-universal.tar.gz";
    hash = "sha256-YcUPGlbLWgaNg87Pt0Ad10orvZdOwnY/dq8iWDgqxVk=";
  };

  sourceRoot = ".";

  installPhase = ''
    install -m 755 -D macicon $out/bin/macicon
  '';

  meta = with pkgs.lib; {
    description = manifest.description;
    homepage = "https://github.com/DarkRader/macicon";
    license = licenses.mit;
    platforms = platforms.darwin;
    mainProgram = "macicon";
  };
}
