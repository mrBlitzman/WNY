const invoke = (cmd, args) => {
  if (window.__TAURI__ && window.__TAURI__.core) {
    return window.__TAURI__.core.invoke(cmd, args);
  }
  return Promise.reject(new Error("tauri missing"));
};

let snap = null;
let wizStep = 0;
let wizLang = "en";
let wizBoot = false;

const $ = (id) => document.getElementById(id);

function t(path) {
  if (!snap) return path;
  const parts = path.split(".");
  let v = snap.catalog;
  for (const p of parts) v = v && v[p];
  return typeof v === "string" ? v : path;
}

function renderMain() {
  $("wizard").classList.add("hidden");
  $("main").classList.remove("hidden");
  $("tagline").textContent = t("tagline");
  $("power").setAttribute("aria-pressed", snap.on ? "true" : "false");
  $("power-label").textContent = snap.on ? t("on") : t("off");
  $("status").textContent = snap.status;
  $("mode-label").textContent = t("mode");
  $("mode-help").textContent = t("modeHelp." + snap.mode);
  $("captive").textContent = t("actions.captive");
  $("ident").textContent = t("actions.newIdentity");
  $("ident").classList.toggle("hidden", snap.mode !== "private");

  const lang = $("lang");
  lang.innerHTML = ["en", "tr", "de"]
    .map((c) => `<option value="${c}" ${c === snap.language ? "selected" : ""}>${c.toUpperCase()}</option>`)
    .join("");

  const mode = $("mode");
  mode.innerHTML = ["protection", "unblock", "private", "emergency"]
    .map((m) => `<option value="${m}" ${m === snap.mode ? "selected" : ""}>${t("modes." + m)}</option>`)
    .join("");

  const slots = [snap.names, snap.filter, snap.road].filter((s) => s.slot !== "filter" || snap.has_filter);
  $("slots").innerHTML = slots
    .map((s) => {
      const opts = [`<option value="auto" ${s.choice === "auto" ? "selected" : ""}>${t("choice.auto")}</option>`,
        `<option value="off" ${s.choice === "off" ? "selected" : ""}>${t("choice.off")}</option>`]
        .concat(
          (s.available || []).map(
            (id) =>
              `<option value="${id}" ${s.engine === id && s.choice === "pin" ? "selected" : ""}>${
                (snap.catalog.engines && snap.catalog.engines[id]) || id
              }</option>`
          )
        )
        .join("");
      return `<article class="slot">
        <div class="slot-top">
          <h2>${t("slots." + s.slot)}</h2>
          <select data-slot="${s.slot}">${opts}</select>
        </div>
        <p class="engine">${s.engine_label}</p>
        <p class="muted small">${t("slotHelp." + s.slot)}</p>
      </article>`;
    })
    .join("");

  $("slots").querySelectorAll("select").forEach((el) => {
    el.onchange = () => invoke("set_slot", { slot: el.dataset.slot, choice: el.value }).then(apply);
  });
}

function renderWizard() {
  $("main").classList.add("hidden");
  $("wizard").classList.remove("hidden");
  const c = snap.catalog.wizard;
  $("wiz-back").classList.toggle("hidden", wizStep === 0);
  $("wiz-next").textContent = wizStep === 3 ? c.done : c.next;
  $("wiz-back").textContent = c.back;
  const body = $("wiz-body");
  if (wizStep === 0) {
    body.innerHTML = `<h2>${c.language}</h2>
      ${["en", "tr", "de"].map((l) => `<label class="wiz-opt"><input type="radio" name="lang" value="${l}" ${wizLang === l ? "checked" : ""}> ${l.toUpperCase()}</label>`).join("")}`;
    body.querySelectorAll("input").forEach((i) => (i.onchange = () => { wizLang = i.value; invoke("set_language", { lang: wizLang }).then(applySilent); }));
  } else if (wizStep === 1) {
    body.innerHTML = `<h2>${c.whatTitle}</h2><p>${c.whatBody}</p>`;
  } else if (wizStep === 2) {
    body.innerHTML = `<h2>${c.permTitle}</h2><p>${c.permBody}</p>`;
  } else {
    body.innerHTML = `<h2>${c.bootTitle}</h2><p>${c.bootBody}</p>
      <label class="wiz-opt"><input type="checkbox" id="boot"> ${c.bootToggle}</label>`;
    $("boot").checked = wizBoot;
    $("boot").onchange = () => { wizBoot = $("boot").checked; };
  }
}

function apply(s) {
  snap = s;
  if (!snap.wizard_done) renderWizard();
  else renderMain();
}

function applySilent(s) {
  snap = s;
}

$("power").onclick = () => invoke("set_on", { on: !snap.on }).then(apply);
$("mode").onchange = () => invoke("set_mode", { mode: $("mode").value }).then(apply);
$("lang").onchange = () => invoke("set_language", { lang: $("lang").value }).then(apply);
$("captive").onclick = () => invoke("captive").then(apply);
$("ident").onclick = () => invoke("new_identity").then(apply);
$("wiz-next").onclick = () => {
  if (wizStep < 3) {
    wizStep += 1;
    renderWizard();
  } else {
    invoke("finish_wizard", { startAtLogin: wizBoot }).then(apply);
  }
};
$("wiz-back").onclick = () => {
  wizStep = Math.max(0, wizStep - 1);
  renderWizard();
};

invoke("snapshot")
  .then(apply)
  .catch(() => {
    document.body.innerHTML = "<p style='padding:24px'>WNY core is not running.</p>";
  });
