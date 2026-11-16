{
  description = "Caskel — Linux-first Rust container runtime";

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
        let caskel = pkgs.callPackage ./nix/package.nix { };
        in {
          inherit caskel;
          default = caskel;
        });

      apps = forAllSystems (pkgs: {
        default = {
          type = "app";
          program = "${self.packages.${pkgs.system}.caskel}/bin/caskel";
        };
      });

      checks = forAllSystems (pkgs: {
        caskel = self.packages.${pkgs.system}.caskel;
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
