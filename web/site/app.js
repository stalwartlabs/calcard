import init, { version, convert } from "./pkg/calcard_web.js";

const $ = (id) => document.getElementById(id);

const SAMPLES = [
  "ical_001.ics",
  "ical_002.ics",
  "ical_003.ics",
  "vcard_001.vcf",
  "vcard_002.vcf",
  "vcard_003.vcf",
  "jscal_001.json",
  "jscal_002.json",
  "jscal_003.json",
  "jscontact_001.json",
  "jscontact_002.json",
];

function showError(message) {
  $("error").textContent = message;
  $("error").hidden = false;
  $("result-card").hidden = true;
  $("occ-card").hidden = true;
}

function clearError() {
  $("error").hidden = true;
}

function renderOccurrences(occurrences) {
  const card = $("occ-card");
  if (!occurrences || occurrences.length === 0) {
    card.hidden = true;
    return;
  }
  $("occ-sub").textContent =
    `First ${occurrences.length} occurrence${occurrences.length === 1 ? "" : "s"} of the pasted calendar event:`;
  const body = $("occ-body");
  body.textContent = "";
  for (const occ of occurrences) {
    const row = document.createElement("tr");
    const from = document.createElement("td");
    const to = document.createElement("td");
    from.textContent = occ.from;
    to.textContent = occ.to;
    row.append(from, to);
    body.appendChild(row);
  }
  card.hidden = false;
}

function runConvert() {
  clearError();
  const input = $("source").value;
  if (input.trim() === "") {
    $("result-card").hidden = true;
    $("occ-card").hidden = true;
    return;
  }

  let result;
  try {
    result = convert(input);
  } catch (err) {
    showError(String(err));
    return;
  }

  if (result.error) {
    showError(result.error);
    return;
  }

  $("result-sub").textContent =
    `Your ${result.source_type} converted to ${result.counterpart}, then back again:`;
  $("conversion-head").textContent = `${result.source_type} → ${result.counterpart}`;
  $("roundtrip-head").textContent = `${result.counterpart} → ${result.source_type}`;
  $("conversion").textContent = result.conversion;
  $("roundtrip").textContent = result.roundtrip;
  $("result-card").hidden = false;

  renderOccurrences(result.occurrences);
}

async function loadSample() {
  const name = SAMPLES[Math.floor(Math.random() * SAMPLES.length)];
  try {
    const res = await fetch(`./samples/${name}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    $("source").value = await res.text();
    runConvert();
  } catch (err) {
    showError(`Could not load sample: ${err}`);
  }
}

function setupCopyButtons() {
  document.querySelectorAll("[data-copy]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const text = $(btn.dataset.copy).textContent;
      try {
        await navigator.clipboard.writeText(text);
        const old = btn.textContent;
        btn.textContent = "Copied";
        setTimeout(() => (btn.textContent = old), 1200);
      } catch (_) {
        /* clipboard blocked */
      }
    });
  });
}

async function boot() {
  const badge = $("wasm-badge");
  try {
    await init();
    const v = version();
    badge.textContent = `calcard v${v}`;
    badge.dataset.state = "ok";
    $("version").textContent = `v${v}`;
  } catch (err) {
    badge.textContent = "wasm failed";
    badge.dataset.state = "err";
    console.error(err);
    return;
  }

  $("convert").addEventListener("click", runConvert);
  $("load-sample").addEventListener("click", loadSample);
  $("source").addEventListener("change", runConvert);
  setupCopyButtons();
}

boot();
