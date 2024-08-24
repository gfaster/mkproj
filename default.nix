{ pkgs ? import <nixpkgs> {} }: with pkgs;
rustPlatform.buildRustPackage rec {
  pname = "mkproj";
  version = "0.1.0";

  src = fetchFromGitHub {
    owner = "gfaster";
    repo = pname;
    rev = "v${version}";
    sha256 = "sha256-NtkEM+uVG2gp6XjdqgRX5Dv5Aup8BQZCaV07IztxsTo=";
  };

  cargoSha256 = "sha256-sn4lbWSd47wWU+wC9SS8VJ4Obgqo1uZZTZ/PWJ4BjwM=";

  meta = with lib; {
    description = "My builder for creating new projects";
    homepage = "https://github.com/gfaster/mkproj";
    license = licenses.gpl3Plus;
    maintainers = [];
  };
}
