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
