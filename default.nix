{
  pkgs,
  stdenv,
  lib,
}: {
  fzf-runner = pkgs.rustPlatform.buildRustPackage {
    pname = "fzf-runner";
    version = "1.0.0";

    src = ./.;
    cargoHash = "sha256-ZLts7ctvoRvIx+NxPOt/Kn0VQdVDQY+o58xRPcdSW18=";
  };
}
