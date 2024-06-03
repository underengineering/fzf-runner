{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    {
      overlays.default = final: prev: {
        fzf-runner = self.packages.${final.system}.fzf-runner;
      };
    }
    // (flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      inherit (pkgs) rustPlatform;
    in {
      packages = rec {
        fzf-runner = rustPlatform.buildRustPackage {
          pname = "fzf-runner";
          version = "1.0.0";

          src = ./.;
          cargoHash = "sha256-zLaXnr6ZVsxA65C4XyOFEzmgj0kS6AItVP1VTnkKRXU=";
        };
        default = fzf-runner;
      };
      devShell = pkgs.mkShell {
        packages = with pkgs; [cargo clippy rust-analyzer rustfmt];
      };
    }));
}
