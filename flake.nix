{
  description = "Kiri container engine — reproducible Linux builds from the fork";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      # Start with native x86-64 Linux, the platform we actually use and test.
      systems = [ "x86_64-linux" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems
        (system: f (import nixpkgs { inherit system; }));
    in
    {
      packages = forAllSystems (pkgs:
        let kiri = pkgs.callPackage ./nix/package.nix { };
        in {
          inherit kiri;
          default = kiri;
        });

      apps = forAllSystems (pkgs: {
        default = {
          type = "app";
          program = "${self.packages.${pkgs.system}.kiri}/bin/kiri";
        };
      });

      checks = forAllSystems (pkgs: {
        kiri = self.packages.${pkgs.system}.kiri;
      });

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            pkg-config
            hyperfine
          ];
        };
      });
    };
}
