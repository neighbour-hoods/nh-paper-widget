{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    node2nix.url = "github:samuelludwig/node2nix";
    holonix = {
      url = "github:holochain/holonix";
      flake = false;
    };
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.url = "github:nix-community/naersk";

    # misc
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, node2nix, holonix, rust-overlay, naersk, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        holonixMain = import holonix {
          holochainVersionId = "v0_0_139";
          include = {
            rust = false;
          };
        };


        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlay ];
        };

        rustVersion = "1.60.0";

        wasmTarget = "wasm32-unknown-unknown";

      in

      with pkgs;
      {
        devShell = pkgs.mkShell {
          inputsFrom = [
            holonixMain.main
          ];

          buildInputs = [
            holonixMain.pkgs.binaryen
          ] ++ (with pkgs; [
            node2nix.defaultPackage.${system}
            nodejs
            nodePackages.webpack
            nodePackages.webpack-cli
            miniserve
            (rust-bin.stable.${rustVersion}.default.override {
              targets = [ wasmTarget ];
            })
          ]);

          shellHook = ''
            export CARGO_HOME=~/.cargo
            export CARGO_TARGET_DIR=target
          '';
        };

        packages.frontend =
          let
            nodeDependencies = (pkgs.callPackage ./default.nix {
              inherit system pkgs;
            }).shell.nodeDependencies;

            dist = stdenv.mkDerivation {
              name = "nh-mvp_js_dist";
              src = ./.;
              buildInputs = [
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
      });
}
