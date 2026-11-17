// parity: linux
// lang: es
// Caskel's visible identity is translated without rewriting legacy protocols.
const R = [];
const t = (label, fn) => { try { fn(); R.push("PASS " + label); } catch (e) { R.push("FAIL " + label + ": " + e.message); } };
t("Linux product branding is Caskel, never a Kiri nav section", () => {
  if (![...document.querySelectorAll("aside .navlabel")].some((x) => x.textContent === "Caskel")) throw new Error("sidebar still not branded");
});
t("Spanish locale retains its words but names Caskel", () => {
  const v = tr("Build with Kiri");
  if (!/Caskel/.test(v) || /\bKiri\b/.test(v)) throw new Error(v);
});
t("search combobox has a Caskel accessible name while preserving its translation key", () => {
  const search = document.querySelector("#gq");
  if (!search || !search.getAttribute("aria-label")?.includes("Caskel")) throw new Error("search still advertises Kiri");
  if (search.getAttribute("data-i18n-label") !== "Search Kiri") throw new Error("legacy translation key was renamed");
});
t("pinned Nix release state never advertises Kiri downloads or automatic updates", () => {
  const oldState = ui.updState, oldUpdate = ui.upd;
  try {
    ui.updState = { state: "nix", current: "0.2.0" };
    ui.upd = null;
    const html = aboutUpdate("0.2.0");
    if (!html.includes("Nix")) throw new Error("pinned Nix identity not shown");
    if (html.includes("data-updcheck") || html.includes("data-updauto") || html.includes("runkiri.com/releases"))
      throw new Error("inherited release actions still visible");
  } finally {
    ui.updState = oldState;
    ui.upd = oldUpdate;
  }
});
t("diagnostic command labels say Caskel without changing translation keys", () => {
  const s = brandedCliLabel(tr("A kiri command"));
  if (!/Caskel/.test(s) || /\bkiri\b/i.test(s)) throw new Error(s);
});
t("Linux diagnostics is localized, names upstream Kiri Cloud and requires opt-in", () => {
  const key = "Kiri Cloud is operated by upstream, not Caskel. Uploading a diagnostic report requires CASKEL_ALLOW_LEGACY_KIRI_CLOUD=1. Review the redacted report before sending.";
  const disclosure = tr(key);
  if (!DICT[key] || disclosure === key) throw new Error("missing Spanish localization");
  if (!disclosure.includes("Kiri Cloud") || !disclosure.includes("CASKEL_ALLOW_LEGACY_KIRI_CLOUD=1"))
    throw new Error("provider or opt-in lost in translation");
  const pane = diagPane();
  if (!pane.includes(disclosure) || !pane.includes(tr("Send to Kiri Cloud")))
    throw new Error("localized disclosure or external recipient button missing");
  if (pane.includes("Send to Caskel")) throw new Error("incorrect report recipient");
  reportDialog("send");
  try {
    const text = document.querySelector("#rp")?.textContent || "";
    if (!text.includes(disclosure)) throw new Error("localized report recipient disclosure missing");
    if (!text.includes(tr("Send to Kiri Cloud"))) throw new Error("report dialog recipient was renamed");
  } finally {
    document.querySelector("#rp [data-close]")?.click();
  }
});
t("Linux save report description is local and never advertises Finder", () => {
  reportDialog("save");
  try {
    const key = "A redacted text file is saved locally. Read it before sharing.";
    const description = tr(key);
    const text = document.querySelector("#rp")?.textContent || "";
    if (description === key || !text.includes(description)) throw new Error("save text was not localized");
    if (text.includes("Finder")) throw new Error("Mac file manager incorrectly advertised");
  } finally {
    document.querySelector("#rp [data-close]")?.click();
  }
});
t("Linux crash card never promises uploads without the external-cloud opt-in", () => {
  const before = ui.crash;
  try {
    ui.crash = { part: "cli", time: "2026-11-16T19:00:00Z", count: 1,
                 crashReports: true, sent: false, hidden: false };
    const html = crashCard();
    if (!html.includes("CASKEL_ALLOW_LEGACY_KIRI_CLOUD=1") || !html.includes("Caskel"))
      throw new Error("crash card has misleading legacy provider status");
  } finally {
    ui.crash = before;
  }
});
t("upstream Kiri Cloud and Kiri Share are not falsely rebranded", () => {
  if (caskelBrand("Kiri Cloud; Kiri Share") !== "Kiri Cloud; Kiri Share") throw new Error("falsely named a Caskel cloud");
});
t("legacy filesystem paths and protocol names are unchanged", () => {
  if (caskelBrand("~/.kiri/ui.json KIRI_SOCKET /run/kiri/docker.sock *.kiri.local") !== "~/.kiri/ui.json KIRI_SOCKET /run/kiri/docker.sock *.kiri.local") throw new Error("mutated compatibility path");
});
t("user-provided workload names are not rewritten", () => {
  const v = tr("Kiri sees {name}", { name: "Kiri" });
  if (v !== "Caskel sees Kiri") throw new Error(v);
});
t("Linux setup describes only the isolated Caskel service and socket", () => {
  onboarding({ complete: false, ready: false, checks: {
    engine: "ok", running: "ok", access: "ok", docker: "off", builds: "off"
  }, skipped: [], user: "qa" });
  const words = document.querySelector("#onb")?.textContent || "";
  if (!words.includes("caskel.service") || !words.includes("/run/caskel/docker.sock")) throw new Error(words.slice(0, 250));
  if (words.includes("kiri.service") || words.includes("/run/kiri/docker.sock")) throw new Error("legacy Kiri paths shown in Caskel onboarding");
  document.querySelector("#onb")?.remove();
  ui.onb = null;
});
report(R);
