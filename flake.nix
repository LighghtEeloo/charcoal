{
  description = "A command line dictionary";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = manifest.name;
          version = manifest.version;

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [
            pkgs.pkg-config
            pkgs.makeWrapper
          ];

          buildInputs = [
            pkgs.openssl
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            pkgs.alsa-lib
          ];

          postInstall = ''
            wrapProgram $out/bin/charcoal \
              --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.didyoumean ]}
          '';

          meta = with pkgs.lib; {
            description = manifest.description;
            homepage = manifest.homepage;
            license = licenses.mit;
            maintainers = [ ];
            mainProgram = "charcoal";
          };
        };

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/charcoal";
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.default ];
          packages = with pkgs; [
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt
          ];
        };
      }
    );
}
