{
  description = "Application to set wallpapers from reddit as desktop-background";
  inputs = {
    os_flake.url = "github:septias/nixos-config";
    nixpkgs.follows = "os_flake/nixpkgs";
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };
  outputs = inputs:
    with inputs;
      flake-utils.lib.eachDefaultSystem (
        system: let
          version = "0.1.3";
          pkgs = import nixpkgs {
            overlays = [(import rust-overlay)];
            inherit system;
          };
          unstable = import nixpkgs-unstable {
            inherit system;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
            gobject-introspection
            cargo
            cargo-tauri
            nodejs
          ];

          buildInputs = with pkgs; [
            at-spi2-atk
            atkmm
            cairo
            gdk-pixbuf
            glib
            gtk3
            harfbuzz
            librsvg
            libsoup_3
            pango
            webkitgtk_4_1
            openssl
            makeWrapper
          ];

          rust-toolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = ["rust-src" "rustfmt" "rust-docs" "clippy" "rust-analyzer"];
          };
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rust-toolchain;
            rustc = rust-toolchain;
          };
          name = "reddit-wallpapers";
          frontend = pkgs.stdenv.mkDerivation (finalAttrs: {
            inherit version;
            pname = "reddit-wallpapers-frontend";
            src = pkgs.lib.cleanSource ./.;
            nativeBuildInputs = with unstable; [
              nodejs
              pnpm.configHook
            ];
            pnpmDeps = unstable.pnpm.fetchDeps {
              inherit (finalAttrs) pname version src;
              hash = "sha256-O6B5Zoc8UJrOtuFtA7SdvX/8RoAZuazPaTwdoIs8jGQ=";
            };

            installPhase = ''
              pnpm build
              cp -r dist $out
            '';
          });
          desktopItem = pkgs.makeDesktopItem {
            name = "Reddit Wallpapers";
            desktopName = "Reddit Wallpapers";
            icon = "reddit-wallpapers";
            comment = "Wallpapers";
            exec = "reddit-wallpapers";
            categories = ["Office"];
          };
          icon = ./src-tauri/icons/icon.png;
          icon-small = ./src-tauri/icons/128x128.png;
        in rec {
          formatter = pkgs.alejandra;
          packages = {
            ${name} = rustPlatform.buildRustPackage rec {
              inherit buildInputs name desktopItem version;
              nativeBuildInputs = buildInputs ++ [pkgs.pkg-config];
              src = ./src-tauri;
              cargoLock = {
                lockFile = ./src-tauri/Cargo.lock;
                outputHashes = {
                  "wallpaper-4.0.0" = "sha256-2t7c+RLmScXH9FoPyTx7fCroWLd3qry7ZT3bGuUNjWA=";
                };
              };

              postPatch = ''
                substituteInPlace tauri.conf.json --replace-fail '"frontendDist": "../dist",' '"frontendDist": "${frontend}",'
              '';

              postInstall = ''
                mkdir -p $out/share/icons/hicolor/128x128/apps
                mkdir -p $out/share/icons/hicolor/512x512/apps
                cp ${icon-small} $out/share/icons/hicolor/128x128/apps/reddit-wallpapers.png
                cp ${icon} $out/share/icons/hicolor/512x512/apps/reddit-wallpapers.png

                mkdir -p "$out/share/applications"
                cp $desktopItem/share/applications/* $out/share/applications

                wrapProgram $out/bin/${name} --prefix PATH : ${pkgs.glib}/bin --set WEBKIT_DISABLE_COMPOSITING_MODE 1
              '';

              meta = {
                description = "Application to set r/wallpapers from reddit as desktop-background";
                homepage = "https://github.com/Septias/reddit-wallpapers";
                mainProgram = "reddit-wallpapers";
              };
            };
            default = packages.${name};
          };
          devShells.default = pkgs.mkShell {
            buildInputs = buildInputs ++ nativeBuildInputs ++ [rust-toolchain pkgs.cargo-tauri];
            RUST_BACKTRACE = 1;

            shellHook = ''
              export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath buildInputs}:$LD_LIBRARY_PATH
              export XDG_DATA_DIRS=${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS
              export WEBKIT_DISABLE_COMPOSITING_MODE=1
            '';
          };
        }
      );
}
