{ pkgs ? import <nixpkgs> {} }: with pkgs;
rustPlatform.buildRustPackage rec {
  pname = "mkproj";
  version = "0.1.1";

  src = fetchFromGitHub {
    owner = "gfaster";
    repo = pname;
    rev = "v${version}";
    sha256 = "";
  };

  cargoSha256 = "sha256-XBMLrSj5kbqBnGSm28Abe1YkqFcgMvxZDVSSmU1cvzg=";

  meta = with lib; {
    description = "My builder for creating new projects";
    homepage = "https://github.com/gfaster/mkproj";
    license = licenses.gpl3Plus;
    maintainers = [];
  };
}
