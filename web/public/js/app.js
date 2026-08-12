/**
 * Landing page — loads release manifest and probes VM worker stub.
 * Does NOT simulate an OS terminal; serial pane reflects worker messages only.
 */

/** Resolve asset URLs for local serve (/) and GitHub Pages (/ather-os/). */
function assetUrl(relativePath) {
  const clean = relativePath.replace(/^\//, "");
  const meta = document.querySelector('meta[name="aether-base"]');
  if (meta?.content) {
    const base = meta.content.endsWith("/") ? meta.content : `${meta.content}/`;
    return `${base}${clean}`;
  }
  const pagesMatch = location.pathname.match(/^(.*\/ather-os)\/?/i);
  if (pagesMatch) {
    return `${pagesMatch[1]}/${clean}`;
  }
  return clean;
}

const bootStatusEl = document.getElementById("boot-status");
const manifestMetaEl = document.getElementById("manifest-meta");
const artifactBodyEl = document.getElementById("artifact-body");
const manifestErrorEl = document.getElementById("manifest-error");
const serialPaneEl = document.getElementById("serial-pane");
const vmProbeBtn = document.getElementById("vm-probe");

function formatBytes(n) {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KiB`;
  return `${(n / (1024 * 1024)).toFixed(2)} MiB`;
}

function renderManifest(manifest) {
  const browser = manifest.boot?.browser_runtime ?? {};
  const status = browser.status ?? "unknown";

  if (status === "not_available") {
    bootStatusEl.textContent =
      "Blocked — UEFI/OVMF required. Local QEMU works; in-browser boot targets qemu.wasm (Phase 2).";
    bootStatusEl.className = "status-blocked";
  } else if (status === "ready") {
    bootStatusEl.textContent = "Ready — emulator can boot verified artifacts.";
    bootStatusEl.className = "status-ready";
  } else {
    bootStatusEl.textContent = `Status: ${status}`;
    bootStatusEl.className = "status-blocked";
  }

  const meta = [
    ["Version", manifest.version],
    ["Git commit", (manifest.git_commit ?? "").slice(0, 12)],
    ["Generated", manifest.generated_at],
    ["Firmware", manifest.boot?.firmware],
    ["Browser target", browser.target ?? "—"],
  ];

  manifestMetaEl.replaceChildren(
    ...meta.flatMap(([label, value]) => {
      const dt = document.createElement("dt");
      dt.textContent = label;
      const dd = document.createElement("dd");
      dd.textContent = value ?? "—";
      return [dt, dd];
    })
  );

  artifactBodyEl.replaceChildren();
  for (const art of manifest.artifacts ?? []) {
    const tr = document.createElement("tr");
    for (const [label, text] of [
      ["Path", art.path],
      ["Role", art.role],
      ["Size", formatBytes(art.size_bytes ?? 0)],
      ["SHA-256", art.sha256 ?? ""],
    ]) {
      const td = document.createElement("td");
      td.dataset.label = label;
      td.textContent = text;
      if (label === "SHA-256") td.className = "sha";
      tr.appendChild(td);
    }
    artifactBodyEl.appendChild(tr);
  }

  vmProbeBtn.disabled = status === "ready" ? false : true;
  vmProbeBtn.title =
    status === "ready"
      ? "Start qemu.wasm worker"
      : "Browser boot blocked until Phase 2 (ADR-0010)";
}

async function loadManifest() {
  try {
    const res = await fetch(assetUrl("manifest.json"), { cache: "no-store" });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const manifest = await res.json();
    renderManifest(manifest);
    manifestErrorEl.hidden = true;
  } catch (err) {
    bootStatusEl.textContent = "Manifest not found — run scripts/build-web-artifacts.ps1";
    bootStatusEl.className = "status-blocked";
    manifestErrorEl.textContent = String(err);
    manifestErrorEl.hidden = false;
  }
}

function appendSerial(line) {
  serialPaneEl.textContent += `\n${line}`;
  serialPaneEl.scrollTop = serialPaneEl.scrollHeight;
}

function probeWorker() {
  if (typeof Worker === "undefined") {
    appendSerial("[vm] Web Workers unavailable");
    return;
  }

  appendSerial("[vm] spawning worker…");
  const worker = new Worker(assetUrl("vm/worker.js"), { type: "module" });

  worker.onmessage = (ev) => {
    const msg = ev.data;
    if (msg?.type === "serial") {
      appendSerial(msg.line);
    } else if (msg?.type === "status") {
      appendSerial(`[vm] ${msg.state}: ${msg.detail ?? ""}`);
    }
  };

  worker.onerror = (err) => {
    appendSerial(`[vm] worker error: ${err.message}`);
  };

  worker.postMessage({ type: "init", manifestUrl: assetUrl("manifest.json") });
}

vmProbeBtn.addEventListener("click", probeWorker);
loadManifest();
