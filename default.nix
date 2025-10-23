{ pkgs ? import <nixpkgs> {} }: with pkgs;
rustPlatform.buildRustPackage rec {
  pname = "mkproj";
  version = "0.1.3";

  src = fetchFromGitHub {
    owner = "gfaster";
    repo = pname;
    rev = "v${version}";
    sha256 = "";
  };
  # src = ./.;

  nativeBuildInputs = [ makeWrapper ];

  postInstall = ''
    wrapProgram $out/bin/mkproj --prefix PATH : ${lib.makeBinPath [ git ]}
  '';

  doCheck = false; # TODO: figure out how to get nix working during check phase

  cargoHash = "sha256-jhyeekmuqaMRG5wc4kW+ADP567dk1p28nwqag6wXy48=";

  meta = with lib; {
    description = "My builder for creating new projects";
    homepage = "https://github.com/gfaster/mkproj";
    license = licenses.gpl3Plus;
    maintainers = [];
  };
}
