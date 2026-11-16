{
  lib,
  rustPlatform,
  pkg-config,
}:

rustPlatform.buildRustPackage {
  pname = "caskel";
  # Upstream Cargo.toml intentionally uses 0.0.0 for checkout builds.
  version = "0.2.1-git";

  src = lib.cleanSource ../.;

  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [ pkg-config ];

  # Build the exact Rust packages inside Nix's sandbox. The Linux-only
  # cargo xtask wrapper invokes this derivation, not the other way around.
  cargoBuildFlags = [
    "-p" "caskel"
    "-p" "caskel-init"
  ];

  # Runtime/integration tests require privileged Linux namespaces, mounts and
  # networking; the initial gate is a reproducible binary build.
  doCheck = false;

  postInstall = ''
    test -x "$out/bin/caskel"
    test -x "$out/bin/caskel-init"
    # Old agent launcher names remain as a compatibility link for persisted
    # runtime integration; new installs expose caskel-init canonically.
    ln -s caskel-init "$out/bin/kiri-init"
    # Opt-in service template; not registered or activated by this build.
    install -Dm444 ${./systemd/caskel.service} "$out/lib/systemd/system/caskel.service"
    test -f "$out/lib/systemd/system/caskel.service"
  '';

  meta = with lib; {
    description = "Linux-first Rust container engine compatible with Docker's API";
    homepage = "https://github.com/sguzman/caskel";
    license = licenses.asl20;
    platforms = [ "x86_64-linux" ];
    mainProgram = "caskel";
  };
}
