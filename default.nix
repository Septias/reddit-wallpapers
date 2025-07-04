{
  lib,
  stdenv,
  rustPlatform,
  cargo-tauri,
  nodejs,
  openssl,
  pkg-config,
  webkitgtk_4_1,
  fetchFromGitHub,
  makeDesktopItem,
  pnpm_9,
  makeWrapper,
  glib,
}: let
  version = "0.1.3";
  pname = "reddit-wallpapers";
  src = fetchFromGitHub {
    owner = "Septias";
    repo = "reddit-wallpapers";
    rev = "5592d181759f1d97bd5edec479a7663718c11d71";
    hash = "sha256-MiiExymLpsGb3d2V3o7K0Sb5bhGW9PbuO3z/a41OGc4=";
  };

  desktopItem = makeDesktopItem {
    name = "Reddit Wallpapers";
    desktopName = "Reddit Wallpapers";
    icon = "reddit-wallpapers";
    comment = "Wallpapers";
    exec = "reddit-wallpapers";
    categories = ["Office"];
  };

  icon = "${src}/src-tauri/icons/icon.png";
  icon-small = "${src}/src-tauri/icons/128x128.png";
in
  rustPlatform.buildRustPackage (finalAttrs: {
    inherit pname desktopItem version src;

    # For the frontend
    pnpmDeps = pnpm_9.fetchDeps {
      inherit (finalAttrs) pname version src;
      hash = "sha256-H4Ux4PjahhYAUGRVzXM5znmSAncXMn5wy96R7jBlHFc=";
    };

    nativeBuildInputs = [
      nodejs
      pnpm_9.configHook
      cargo-tauri.hook
      pnpm_9
      pkg-config
    ];

    buildInputs = [
      glib
      webkitgtk_4_1
      openssl
      makeWrapper
    ];

    # For the backend
    cargoRoot = "src-tauri";
    cargoLock = {
      lockFile = "${src}/src-tauri/Cargo.lock";
      outputHashes = {
        "wallpaper-4.0.0" = "sha256-2t7c+RLmScXH9FoPyTx7fCroWLd3qry7ZT3bGuUNjWA=";
      };
    };
    buildAndTestSubdir = finalAttrs.cargoRoot;

    postInstall = ''
      mkdir -p $out/share/icons/hicolor/128x128/apps
      mkdir -p $out/share/icons/hicolor/512x512/apps
      cp ${icon-small} $out/share/icons/hicolor/128x128/apps/reddit-wallpapers.png
      cp ${icon} $out/share/icons/hicolor/512x512/apps/reddit-wallpapers.png

      mkdir -p "$out/share/applications"
      cp $desktopItem/share/applications/* $out/share/applications
    '';

    meta = {
      description = "Application to set r/wallpapers from reddit as desktop-background";
      homepage = "https://github.com/Septias/reddit-wallpapers";
      mainProgram = "reddit-wallpapers";
    };
  })
