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
- Do not click the UI's setup/fix, restart, delete, auto-reporting, or cloud
  send buttons. Some features are deliberately unavailable on Linux.
- The UI can read engine status and may write UI preferences/diagnostic
  markers under the current user's Caskel state directory. Do not treat
  starting a GUI as a strictly read-only operation.
- Leave `CASKEL_ALLOW_LEGACY_KIRI_CLOUD` unset for this test. Do not submit
  any diagnostic report to upstream Kiri Cloud.
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
4. In Settings → Diagnostics, the external recipient is clearly **Kiri
   Cloud** (not a Caskel cloud), and the explicit
   `CASKEL_ALLOW_LEGACY_KIRI_CLOUD=1` prerequisite is visible before upload.
   **Do not actually send**.
5. Inspect the report dialog in **save** and **send** modes, cancelling each
   without writing or uploading: the save copy must not promise macOS Finder
   or a fixed Desktop path; the send copy names Kiri Cloud, not Caskel.
6. If practical, switch the UI language to Spanish, then an RTL locale
   (Arabic/Persian/Central Kurdish) and a CJK locale. Ensure the report
   disclosure, ID placeholders and opt-in variable remain readable and no
   message falls back to unlocalized English. Switching a language **writes
   a UI preference**; skip this step if that is unacceptable for the host.
7. Record build output, compositor/window behavior, screenshots (with no
   hostnames, tokens, credentials or private container details), and any
   failed steps. Do not mark them passed without observing them.

## Definition of done

All 37 translation catalogs pass key/placeholder parity and the UI
regressions; Nix/Rust, GTK compilation, CodeQL and performance CI pass on
the final commit; **and** the desktop session above is actually exercised.
Passing only the first two conditions leaves native desktop QA pending.
