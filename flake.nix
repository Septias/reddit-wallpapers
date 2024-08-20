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
    flake-utils.follows = "rust-overlay/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };
  outputs = inputs:
    with inputs;
      flake-utils.lib.eachDefaultSystem (
        system: let
          version = "0.1.2";

          pkgs = import nixpkgs {
            overlays = [(import rust-overlay)];
            inherit system;
          };
          unstable = import nixpkgs-unstable {
            inherit system;
          };
          libraries = with pkgs; [
            webkitgtk
            gtk3
            cairo
            gdk-pixbuf
            glib
            dbus
            openssl_3
            librsvg
          ];

          buildInputs = with pkgs; [
            pkg-config
            dbus
            openssl_3
            glib
            gtk3
            libsoup
            webkitgtk
            librsvg
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
              unstable.pnpm.configHook
            ];
            pnpmDeps = unstable.pnpm.fetchDeps {
              inherit (finalAttrs) pname version src;
              hash = "sha256-OsCughjP93BfcxyuNt2EnqwZvyLCEvVSbJeiOFGKJIo=";
            };

            installPhase = ''
              pnpm build
              cp -r dist $out
            '';
          });
          desktopItem = pkgs.makeDesktopItem {
            name = "Reddit Wallpapers";
            desktopName = "Reddit Wallapapers";
            icon = "reddit-wallpapers";
            comment = "Wallpapers";
            exec = "reddit-wallpapers";
            categories = [ "Office" ];
          };
          icon = ./src-tauri/icons/icon.png;
          icon-small = ./src-tauri/icons/128x128.png;
        
        in rec {
          formatter = pkgs.alejandra;
          packages = {
            ${name} = rustPlatform.buildRustPackage rec {
              inherit buildInputs name desktopItem version;
              nativeBuildInputs = buildInputs;
              src = ./src-tauri;
              cargoLock = {
                lockFile = ./src-tauri/Cargo.lock;
                outputHashes = {
                  "wallpaper-4.0.0" = "sha256-C65jjr0dEGb52YcMLwCcrT4Wqf+xZN8eGtp8sXFF7fE=";
                };
              };

              postPatch = ''
                substituteInPlace tauri.conf.json --replace '"distDir": "../dist",' '"distDir": "${frontend}",'
              '';
      
              postInstall = ''
                mkdir -p $out/share/icons/hicolor/128x128/apps
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
            buildInputs = buildInputs ++ [rust-toolchain pkgs.cargo-tauri];
            RUST_BACKTRACE = 1;

            shellHook = ''
              export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH
              export XDG_DATA_DIRS=${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS
              export WEBKIT_DISABLE_COMPOSITING_MODE=1 
            '';
          };
        }
      );
}
