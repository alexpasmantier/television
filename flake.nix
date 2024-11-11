{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    nci = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {
    parts,
    nci,
    ...
  }:
    parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux"];
      imports = [nci.flakeModule ./crates.nix];
      perSystem = {config, ...}: let
        outputs = config.nci.outputs;
        package =
          outputs."television".packages.release;
      in {
        devShells.default = outputs."television".devShell;
        packages.default = package;
        apps.default = {
          program = "${config.packages.default}/bin/tv";
        };
      };
    };
}
