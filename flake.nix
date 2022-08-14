{
  inputs = {
    nh-nix-env.url = "github:neighbour-hoods/nh-nix-env";
    social_sensemaker.url = "github:neighbour-hoods/social_sensemaker";
  };

  outputs = { nh-nix-env, social_sensemaker, ... }:
    let
      flake-utils = nh-nix-env.metavalues.flake-utils;
      nh-supported-systems = nh-nix-env.metavalues.nh-supported-systems;
      rustVersion = nh-nix-env.metavalues.rustVersion;
      naersk = nh-nix-env.metavalues.naersk;
      wasmTarget = nh-nix-env.metavalues.wasmTarget;
      holonixMain = nh-nix-env.metavalues.holonixMain;
    in
    flake-utils.lib.eachSystem nh-supported-systems (system:
      let
        pkgs = nh-nix-env.values.${system}.pkgs;
      in

      {
        devShell = nh-nix-env.shells.${system}.holochainDevShell {
          extraBuildInputs = [
            pkgs.nodejs
            pkgs.nodePackages.webpack
            pkgs.nodePackages.webpack-cli
          ];
        };

        packages.social_sensemaker = social_sensemaker.packages.${system}.social_sensemaker-naersk;
      });
}
