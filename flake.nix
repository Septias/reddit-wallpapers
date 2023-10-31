{
  description = "Rust example flake for Zero to Nix";
  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = { self, nixpkgs, rust-overlay }:
    let
      # Systems supported
      allSystems = [
        "x86_64-linux" # 64-bit Intel/AMD Linux
        "aarch64-linux" # 64-bit ARM Linux
        "x86_64-darwin" # 64-bit Intel macOS
        "aarch64-darwin" # 64-bit ARM macOS
      ];

      # Helper to provide system-specific attributes
      forAllSystems = f: nixpkgs.lib.genAttrs allSystems (system: f {
        pkgs = import nixpkgs { inherit system; };
      });
    in
    {
      packages = forAllSystems ({ pkgs }: {
        default = pkgs.rustPlatform.buildRustPackage {
          name = "zero-to-nix-rust";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };
      });
      devShells.default = pkgs.mkShell {  
        inherit buildInputs LD_LIBRARY_PATH CARGO_NET_GIT_FETCH_WITH_CLI;  
        nativeBuildInputs = nativeBuildInputs ++ [ rust-toolchain pkgs.rust-analyzer ];  
        RUST_BACKTRACE = 1;  
      };  
    };
}