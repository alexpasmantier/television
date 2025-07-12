{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    nixpkgs-mozilla = {
      url = "github:mozilla/nixpkgs-mozilla";
      flake = false;
    };

    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = {
    self,
    flake-utils,
    naersk,
    nixpkgs,
    nixpkgs-mozilla,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = (import nixpkgs) {
          inherit system;

          overlays = [
            (import nixpkgs-mozilla)
          ];
        };

        toolchain =
          (
            pkgs.rustChannelOf
            {
              rustToolchain = ./rust-toolchain.toml;
              sha256 = "KUm16pHj+cRedf8vxs/Hd2YWxpOrWZ7UOrwhILdSJBU=";
            }
          )
          .rust;

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };
      in {
        packages.default = naersk'.buildPackage {
          src = ./.;
          meta = {
            mainProgram = "tv";
          };
        };
        apps = {
          default = flake-utils.lib.mkApp {
            drv = self.packages.${system}.default;
            exePath = "/bin/tv";
          };
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [toolchain];
          packages = with pkgs; [
            rustfmt
            clippy
            rust-analyzer
          ];
        };
      }
    );
}
