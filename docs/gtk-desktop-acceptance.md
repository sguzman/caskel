# Caskel native GTK desktop branding acceptance (Linux)

This is the last *human desktop* branding gate. It is **not** equivalent to the
GitHub Actions headless Chrome tests or the Linux GTK compile check. Record
what actually happens on a Linux desktop before closing the branding-first
gate in `docs/roadmap.md`.

## Safety boundary

- Use a developer checkout and the optional window build, **not** the
  headless Nix package. Do **not** run `caskel ui --start`.
- Do not change Docker's default context or socket, production containers,
  systemd units, privileged group membership, or old Kiri state for this QA.
- Do not click setup/fix, restart, delete, or other host-mutating actions.
  Cloud sharing and report uploads must not be presented at all on Linux.
- The UI can read engine status and may write UI preferences/diagnostic
  markers under the current user's Caskel state directory. Do not treat
  starting a GUI as a strictly read-only operation.
- Do not submit diagnostic reports to any remote service. Old cloud opt-in
  variables must not enable uploads.
- Stop and record a failure rather than attempting privileged fixes.

## Preflight and window build (fish, EndeavourOS/Arch)

In a **clean Caskel developer checkout** on the intended Linux desktop,
check the system's optional GTK 3 and WebKitGTK 4.1 dependencies:

```fish
pkg-config --modversion gtk+-3.0 webkit2gtk-4.1 javascriptcoregtk-4.1
```

If a library is missing, report the exact error before installing or
changing anything. The headless pinned Nix flake does not include the GTK
window as a normal package output.

Build only the optional window-capable binary (the Nix shell provides the
Rust toolchain; the GTK libraries are supplied by the host distribution):

```fish
nix develop --command env PKG_CONFIG_PATH=/usr/lib/pkgconfig:/usr/share/pkgconfig PKG_CONFIG_ALLOW_SYSTEM_CFLAGS=1 PKG_CONFIG_ALLOW_SYSTEM_LIBS=1 cargo build --locked -p caskel --features window
```

Without `--start`, open that binary directly from the checkout:

```fish
env -u CASKEL_ALLOW_LEGACY_KIRI_CLOUD ./target/debug/caskel ui
```

The command may show the isolated-service onboarding screen instead of the
main screen if no Caskel service is installed or accessible. **Do not bypass
this by changing the host.** Record that outcome separately; it is not
evidence of a complete GUI acceptance session.

## Observe, do not mutate

1. Window title and icon are **Caskel**, using the original
   `brand/caskel-mark.svg`; no upstream Kiri trademark art appears.
2. Light/dark themes render correctly on the **real Hyprland session**, and
   the window closes normally without changing a running engine.
3. When Settings → About is accessible, the installed version is shown as
   **Nix-managed**. No upstream Kiri release feed, `.deb`/`.rpm` download,
   automatic updater toggle, or upstream site is presented as Caskel's.
4. In Settings → Diagnostics, **Create report…** remains, and there are no
   cloud recipient, upload, auto-send or send-crash controls.
5. Inspect the **local save** report dialog without writing a report.
   No macOS Finder or guaranteed Desktop path should be advertised.
6. Settings → Resources shows configured CPU and memory values read-only, and
   does not offer VM restarts or resource-save controls. The Network pane
   must not claim Linux container traffic leaves through macOS.
7. The Share navigation entry is absent on Linux. An old configured cloud
   token or `CASKEL_ALLOW_LEGACY_KIRI_CLOUD=1` cannot override the
   networking prohibition; check with source/CLI tests, not live uploads.
8. If practical, switch the UI language to Spanish, then an RTL locale
   (Arabic/Persian/Central Kurdish) and a CJK locale. Ensure local report
   wording stays readable and does not fall back to English. Language changes
   write a local UI preference.
9. Record build output, compositor/window behavior, screenshots (with no
   hostnames, tokens, credentials or private container details), and any
   failed steps. Do not mark them passed without observing them.

## Definition of done

All 37 translation catalogs pass key/placeholder parity and the UI
regressions; Nix/Rust, GTK compilation, CodeQL and performance CI pass on
the final commit; **and** the desktop session above is actually exercised.
Passing only the first two conditions leaves native desktop QA pending.
