{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }: 
  let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};

    nativeBuildInputs = with pkgs; [ rustup gdb ];
    buildInputs = [  ];

    cargoTOML = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    pname = cargoTOML.package.name;
    version = cargoTOML.package.version;

    allBuildInputs = buildInputs;
  in {
    devShells.${system}.default = pkgs.mkShell {
      inherit nativeBuildInputs buildInputs;
      C_INCLUDE_PATH = pkgs.lib.makeIncludePath buildInputs;
      LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath allBuildInputs}";
      LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.libclang ];
      EDITOR = "nvim";
    };

    packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
      inherit pname version buildInputs;
      nativeBuildInputs = [
        pkgs.rustc
        pkgs.cargo
      ];

      C_INCLUDE_PATH = pkgs.lib.makeIncludePath buildInputs;
      LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
      LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.libclang ];

      #postFixup = '' 
      #  patchelf --set-rpath ${pkgs.lib.makeLibraryPath buildInputs} $out/bin/${pname}
      #'';

      cargoLock = {
        outputHashes = {
        };
        lockFile = ./Cargo.lock;
      };

      src = ./.;
    };
  };
}
