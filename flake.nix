{
  description = "Application to set wallpapers from reddit as desktop-background";
  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.follows = "rust-overlay/flake-utils";
    nixpkgs.follows = "rust-overlay/nixpkgs";
    naersk.url = "github:nix-community/naersk";
  };
  outputs = inputs: with inputs; flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs { overlays = [ (import rust-overlay) ]; inherit system; };
      naerskLib = pkgs.callPackage naersk {
        cargo = rust-toolchain;
        rustc = rust-toolchain;
      };
      buildInputs = with pkgs; [ openssl cargo-tauri];
      nativeBuildInputs = with pkgs; [ pkg-config dbus openssl freetype libsoup gtk3 webkitgtk librsvg];
      rust-toolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = [ "rust-src" "rustfmt" "rust-docs" "clippy" "rust-analyzer"];
      };
      name = "reddit-wallpapers";
    in rec {
      packages = {
        ${name} = naerskLib.buildPackage {
          inherit name buildInputs nativeBuildInputs;
          src = ./src-tauri;
          
          postPatch = ''
            echo "hi"
            ls
            substituteInPlace tauri.conf.json --replace '"distDir": "../dist",' '"distDir": "dist",'
          '';
 
          meta = {
            description = "Application to set wallpapers from reddit as desktop-background";
            homepage = "https://github.com/Septias/reddit-wallpapers";
          };
        };
        default = packages.${name};
      };
      devShells.default = pkgs.mkShell {
        inherit buildInputs;
        nativeBuildInputs = nativeBuildInputs ++ [ rust-toolchain ];
        RUST_BACKTRACE = 1;
      };
    }
  );
}
