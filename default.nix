{ pkgs ? import <nixpkgs> {} }: with pkgs;
rustPlatform.buildRustPackage rec {
  pname = "mkproj";
  version = "0.1.1";

  # src = fetchFromGitHub {
  #   owner = "gfaster";
  #   repo = pname;
  #   rev = "v${version}";
  #   sha256 = "";
  # };
  src = ./.;

  nativeBuildInputs = [ makeWrapper ];

  postInstall = ''
    wrapProgram $out/bin/mkproj --prefix PATH : ${lib.makeBinPath [ git ]}
  '';

  doCheck = false; # TODO: figure out how to get nix working during check phase

  # Need to add to check inputs since it doesn't use the binary
  nativeCheckInputs = [ git nix ];

  checkFlags = "--skip shell_is_ok";

  cargoHash = "sha256-w0d9fKruYB7E7vaPUXANar0C6fjNAFQ+wn6KmdXjp7w=";

  meta = with lib; {
    description = "My builder for creating new projects";
    homepage = "https://github.com/gfaster/mkproj";
    license = licenses.gpl3Plus;
    maintainers = [];
  };
}
