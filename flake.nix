{
  inputs = {
    nh-nix-env.url = "github:neighbour-hoods/nh-nix-env";
    node2nix.url = "github:samuelludwig/node2nix";
    social_sensemaker.url = "github:neighbour-hoods/social_sensemaker";
  };

  outputs = { nh-nix-env, node2nix, social_sensemaker, ... }:
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
            node2nix.defaultPackage.${system}
            pkgs.nodejs
            pkgs.nodePackages.webpack
            pkgs.nodePackages.webpack-cli
          ];
        };

        packages.frontend =
          let
            nodeDependencies = (pkgs.callPackage ./default.nix {
              inherit system pkgs;
            }).shell.nodeDependencies;

            dist = pkgs.stdenv.mkDerivation {
              name = "nh-mvp_js_dist";
              src = ./.;
              buildInputs = with pkgs; [
                nodejs
                nodePackages.webpack
                nodePackages.webpack-cli
              ];
              buildPhase = ''
                ln -s ${nodeDependencies}/lib/node_modules ./node_modules
                export PATH="${nodeDependencies}/bin:$PATH"

                cp -r ${./js} .

                npm run fe:build-prod
              '';
              installPhase = ''
                cp -r dist $out/
              '';
            };

          in

          pkgs.stdenv.mkDerivation {
            name = "nh-mvp_frontend";
            buildInputs = [ ];
            unpackPhase = "true";
            installPhase = ''
              mkdir $out
              cp -r ${dist} $out/dist
              cp -r ${./imgs} $out/imgs
              cp -r ${./fonts} $out/fonts
              cp ${./index.html} $out/index.html
              cp ${./style.css} $out/style.css
              cp ${./favicon.ico} $out/favicon.ico
            '';
          };

        packages.paperz-naersk =
          let
            rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
              targets = [ wasmTarget ];
            };

            naersk' = pkgs.callPackage naersk {
              cargo = rust;
              rustc = rust;
            };

            paperz-wasm = naersk'.buildPackage {
              src = ./.;
              copyLibs = true;
              CARGO_BUILD_TARGET = wasmTarget;
              cargoBuildOptions = (opts: opts ++ ["--package=paperz"]);
            };

          in

          pkgs.stdenv.mkDerivation {
            name = "paperz-happ";
            buildInputs = [
              holonixMain.pkgs.holochainBinaries.hc
            ];
            unpackPhase = "true";
            installPhase = ''
              mkdir $out
              cp ${paperz-wasm}/lib/paperz.wasm $out
              cp ${happs/paperz/dna.yaml} $out/dna.yaml
              cp ${happs/paperz/happ.yaml} $out/happ.yaml
              hc dna pack $out
              hc app pack $out
            '';
          };

        packages.social_sensemaker = social_sensemaker.packages.${system}.social_sensemaker-naersk;
      });
}
