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
report(R);
