{
  lib,
  rustPlatform,
  pkg-config,
}:

rustPlatform.buildRustPackage {
  pname = "kiri";
  # Upstream Cargo.toml intentionally uses 0.0.0 for checkout builds.
  version = "0.2.1-git";

  src = lib.cleanSource ../.;

  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [ pkg-config ];

  # Do not use cargo xtask linux: it invokes Docker and downloads packages
  # dynamically, which would defeat Nix's sandboxed build model.
  cargoBuildFlags = [
    "-p" "kiri"
    "-p" "kiri-init"
  ];

  # Runtime/integration tests require privileged Linux namespaces, mounts and
  # networking; the initial gate is a reproducible binary build.
  doCheck = false;

  postInstall = ''
    test -x "$out/bin/kiri"
    test -x "$out/bin/kiri-init"
  '';

  meta = with lib; {
    description = "Rust-based container engine compatible with Docker's API";
    homepage = "https://github.com/libracoder/kiri";
    license = licenses.asl20;
    platforms = [ "x86_64-linux" ];
    mainProgram = "kiri";
  };
}
