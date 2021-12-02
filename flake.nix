{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    node2nix.url = "github:samuelludwig/node2nix";
    holonix = {
      url = "github:holochain/holonix";
      flake = false;
    };

    # misc
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, node2nix, holonix, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        holonixMain = import holonix { };

        pkgs = import nixpkgs {
          inherit system;
        };

        rustVersion = "1.55.0";
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

                npm run build-prod
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
              cp ${./index.html} $out/index.html
              cp ${./style.css} $out/style.css
              cp ${./favicon.ico} $out/favicon.ico
            '';
          };
      });
}
