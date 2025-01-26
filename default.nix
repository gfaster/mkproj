{ pkgs ? import <nixpkgs> {} }: with pkgs;
rustPlatform.buildRustPackage rec {
  pname = "mkproj";
  version = "0.1.1";

  src = fetchFromGitHub {
    owner = "gfaster";
    repo = pname;
    rev = "v${version}";
    sha256 = "sha256-fk+XbP/Emu5u6uWi/G7wwaFpxTwQBfKnyqeSSDZsY7Y=";
  };

  checkFlags = "--skip shell_is_ok";

  cargoSha256 = "sha256-nqsh9XjYdcR5fBxZaFKu25rv2ge5A/9nKzyEJXqKqtw=";

  meta = with lib; {
    description = "My builder for creating new projects";
    homepage = "https://github.com/gfaster/mkproj";
    license = licenses.gpl3Plus;
    maintainers = [];
  };
}
