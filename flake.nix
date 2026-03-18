{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable";
    naersk.url = "github:nix-community/naersk";
  };

  outputs =
    { nixpkgs, naersk, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      naerskLib = pkgs.callPackage naersk { };

      dlopenLibraries = [
        pkgs.libxkbcommon
        pkgs.vulkan-loader
        pkgs.libGL
        pkgs.wayland
      ];

      xeditor = naerskLib.buildPackage {
        src = ./.;
        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs = dlopenLibraries;
      };
    in
    {
      packages.${system}.default = pkgs.symlinkJoin {
        name = "xeditor";
        paths = [ xeditor ];
        buildInputs = [ pkgs.makeWrapper ];
        postBuild = ''
          wrapProgram $out/bin/xeditor \
            --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath dlopenLibraries}"
        '';
      };

      devShells.${system}.default = pkgs.mkShell {
        packages = [
          pkgs.rustc
          pkgs.cargo
          pkgs.clippy
          pkgs.rustfmt
          pkgs.openssl
          pkgs.cargo-watch
          pkgs.rust-analyzer
          pkgs.zenity
        ];
        RUST_LOG = "debug";
        nativeBuildInputs = [ pkgs.pkg-config ];
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath dlopenLibraries;
      };

      formatter = pkgs.rustfmt;
    };
}
