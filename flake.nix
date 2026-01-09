{
  description = "A very fast, portable and hackable fuzzy finder for the terminal";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      flake-parts,
      crane,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = inputs.nixpkgs.lib.systems.flakeExposed;

      perSystem =
        {
          self',
          pkgs,
          lib,
          ...
        }:
        let
          rustBin = inputs.rust-overlay.lib.mkRustBin { } pkgs;
          rustToolchain = rustBin.fromRustupToolchainFile ./rust-toolchain.toml;
          tvCargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

          unfilteredRoot = ./.;
          src = lib.fileset.toSource {
            root = unfilteredRoot;
            fileset = lib.fileset.unions [
              # Default files from crane (Rust and cargo files)
              (craneLib.fileset.commonCargoSources unfilteredRoot)
              # Include shell completions
              (lib.fileset.maybeMissing ./television/utils/shell)
              # Pass tests/config/cli_overrides.rs tests
              (lib.fileset.maybeMissing ./working-dir-test.txt)
              # Pass tests/app.rs tests
              (lib.fileset.maybeMissing ./tests/target_dir)
            ];
          };

          runtimeDeps = with pkgs; [
            bat
            fd
            jq
            just
          ];

          pname = "television";
          version = tvCargo.package.version + "-nightly";
          meta = {
            description = "A very fast, portable and hackable fuzzy finder for the terminal";
            homepage = "https://github.com/alexpasmantier/television";
            license = lib.licenses.mit;
            platforms = lib.platforms.unix;
            mainProgram = "tv";
            maintainers = [
              lib.maintainers.doprz
              "tukanoidd"
            ];
          };

          # Common arguments can be set here to avoid repeating them later
          # Note: changes here will rebuild all dependency crates
          commonArgs = {
            inherit
              src
              pname
              version
              meta
              ;
            strictDeps = true;
            doCheck = false; # NOTE: tests/common/mod.rs:139 fails

            nativeBuildInputs = with pkgs; [
              makeWrapper
              installShellFiles
            ];

            buildInputs =
              [ ]
              ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
                # Additional darwin specific inputs can be set here
                pkgs.libiconv
              ];
          };

          tv = craneLib.buildPackage (
            commonArgs
            // {
              cargoArtifacts = craneLib.buildDepsOnly commonArgs;

              # Additional environment variables or build phases/hooks can be set
              # here *without* rebuilding all dependency crates
              # MY_CUSTOM_VAR = "some value";

              # Wrap the binary to include runtime dependencies in PATH and install shell completions
              postInstall = ''
                wrapProgram $out/bin/tv \
                  --prefix PATH : ${lib.makeBinPath runtimeDeps} \

                installShellCompletion --cmd tv \
                  television/utils/shell/completion.bash \
                  television/utils/shell/completion.zsh \
                  television/utils/shell/completion.fish \
                  television/utils/shell/completion.nu
              '';

            }
          );
        in
        {
          checks = {
            inherit tv;
          };

          packages.default = tv;

          apps.default = {
            inherit meta;
            type = "app";
            program = lib.getExe tv;
          };

          devShells.default = craneLib.devShell {
            name = "tv-dev";
            checks = self'.checks;

            # cargo and rustc are provided by default
            # includes components defined in rust-toolchain.toml
            packages = [ ] ++ runtimeDeps;
          };
        };
    };
}
