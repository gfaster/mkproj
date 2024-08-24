{ pkgs ? import <nixpkgs> {} }:
pkgs.rustPlatform.buildRustPackage rec {
  pname = "mkproj";
  version = "0.1.0";

  src = fetchFromGitHub {
    owner = "gfaster";
    repo = pname;
    rev = version;
    sha256 = "";
  };

  cargoSha256 = "";

  meta = with stdenv.lib; {
    description = "My builder for creating new projects";
    homepage = "https://github.com/gfaster/mkproj";
    license = licenses.gpl3Plus;
    maintainers = [];
  };
}
