{ pkgs ? import <nixpkgs> {} }: with pkgs;
rustPlatform.buildRustPackage rec {
  pname = "mkproj";
  version = "0.1.2";

  src = fetchFromGitHub {
    owner = "gfaster";
    repo = pname;
    rev = "v${version}";
    sha256 = "sha256-QZlo/zbD6egACT2NyL04vuO3rYW1ui773ma+VyCuAPU=";
  };
  # src = ./.;

  nativeBuildInputs = [ makeWrapper ];

  postInstall = ''
    wrapProgram $out/bin/mkproj --prefix PATH : ${lib.makeBinPath [ git ]}
  '';

  doCheck = false; # TODO: figure out how to get nix working during check phase

  # Need to add to check inputs since it doesn't use the binary
  nativeCheckInputs = [ git nix ];

  checkFlags = "--skip shell_is_ok";

  cargoHash = "sha256-85GcC/h5z0gz7GEs1WUh5/nyjHMPaywKzLGudcoqnsE=";

  meta = with lib; {
    description = "My builder for creating new projects";
    homepage = "https://github.com/gfaster/mkproj";
    license = licenses.gpl3Plus;
    maintainers = [];
  };
}
