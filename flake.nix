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

        # GPU backend
        pkgs.vulkan-loader
        pkgs.libGL

        # Window system
        pkgs.wayland
        # pkgs.xorg.libX11
        # pkgs.xorg.libXcursor
        # pkgs.xorg.libXi
      ];
    in
    {
      packages.${system}.default = naerskLib.buildPackage {
        src = ./.;
        nativeBuildInputs = [
          pkgs.pkg-config
        ];
      };
      devShells.${system}.default = pkgs.mkShell {
        # development tools for editing, testing and watching
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
        postInstall = ''
          mkdir -p $out/share/applications
          cp assets/your-app.desktop $out/share/applications/
        '';
        # env.RUSTFLAGS = "-C link-arg=-Wl,-rpath,${nixpkgs.lib.makeLibraryPath dlopenLibraries}";
      };
      formatter = pkgs.rustfmt;
    };
}
