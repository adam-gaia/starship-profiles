{
  inputs,
  pkgs,
  ...
}: let
  lib = pkgs.lib;

  crateBuilder = inputs.self.lib.mkCrateBuilder pkgs;
  commonArgs = crateBuilder.commonArgs;
  cargoArtifacts = crateBuilder.cargoArtifacts;
  craneLib = crateBuilder.craneLib;

  # Build the actual crate itself, reusing the dependency
  # artifacts from above.
  crate = craneLib.buildPackage (
    commonArgs
    // {
      inherit cargoArtifacts;
      doCheck = false; # Don't run tests as part of the build. We run tests with 'nix flake check'

      postInstall = ''
        wrapProgram $out/bin/starship \
          --prefix PATH : ${pkgs.starship}/bin
        ln -s $out/bin/starship $out/bin/starship-profiles
      '';
    }
  );

  fixed = lib.recursiveUpdate crate {
    meta = {
      mainProgram = "starship";
    };
  };
in
  fixed
