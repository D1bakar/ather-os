/**
 * Aether Universal Platform — boot demo UI.
 */

import { startAetherBoot } from "./boot.js";

/** Resolve asset URLs for local serve (/) and GitHub Pages (/ather-os/). */
export function assetUrl(relativePath) {
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

const phaseStatusEl = document.getElementById("phase-status");
const progressBarEl = document.getElementById("progress-bar");
const progressLabelEl = document.getElementById("progress-label");
const bootBtnEl = document.getElementById("boot-btn");
const retryBtnEl = document.getElementById("retry-btn");
const serialPaneEl = document.getElementById("serial-pane");
const manifestMetaEl = document.getElementById("manifest-meta");
const artifactBodyEl = document.getElementById("artifact-body");
const manifestErrorEl = document.getElementById("manifest-error");
const errorPanelEl = document.getElementById("error-panel");
const errorDetailEl = document.getElementById("error-detail");
const bootHintEl = document.getElementById("boot-hint");

function formatBytes(n) {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KiB`;
  return `${(n / (1024 * 1024)).toFixed(2)} MiB`;
}

function setPhase(state, detail) {
  phaseStatusEl.textContent = detail ? `${state} — ${detail}` : state;
  phaseStatusEl.dataset.state = state.toLowerCase().replace(/\s+/g, "-");
}

function setProgress(loaded, total) {
  if (!total) {
    progressBarEl.style.width = "0%";
    progressLabelEl.textContent = "Downloading…";
    return;
  }
  const pct = Math.min(100, Math.round((loaded / total) * 100));
  progressBarEl.style.width = `${pct}%`;
  progressLabelEl.textContent = `${formatBytes(loaded)} / ${formatBytes(total)} (${pct}%)`;
}

function appendSerial(line) {
  if (!line) return;
  if (serialPaneEl.textContent.startsWith("Press BOOT")) {
    serialPaneEl.textContent = "";
  }
  serialPaneEl.textContent += `${line}\n`;
  serialPaneEl.scrollTop = serialPaneEl.scrollHeight;
}

function showError(err, phase) {
  errorPanelEl.hidden = false;
  errorDetailEl.textContent = `[${phase}] ${err.message}`;
  bootBtnEl.disabled = false;
}

function hideError() {
  errorPanelEl.hidden = true;
  errorDetailEl.textContent = "";
}

function renderManifest(manifest) {
  const browser = manifest.boot?.browser_runtime ?? {};
  const status = browser.status ?? "unknown";

  const meta = [
    ["Version", manifest.version],
    ["Git commit", (manifest.git_commit ?? "").slice(0, 12)],
    ["Browser boot", status],
    ["Emulator", browser.qemu?.version ?? browser.target ?? "—"],
    ["Firmware", manifest.boot?.firmware ?? "uefi"],
  ];

  manifestMetaEl.replaceChildren(
    ...meta.flatMap(([label, value]) => {
      const dt = document.createElement("dt");
      dt.textContent = label;
      const dd = document.createElement("dd");
      dd.textContent = value ?? "—";
      if (label === "Browser boot" && status === "ready") {
        dd.className = "status-ready";
      } else if (label === "Browser boot" && status !== "ready") {
        dd.className = "status-blocked";
      }
      return [dt, dd];
    })
  );

  const allArts = [
    ...(manifest.artifacts ?? []),
    ...(manifest.optional_artifacts ?? []),
  ];
  if (manifest.boot?.browser_runtime?.firmware) {
    for (const entry of Object.values(manifest.boot.browser_runtime.firmware)) {
      if (entry?.path) allArts.push(entry);
    }
  }

  artifactBodyEl.replaceChildren();
  for (const art of allArts) {
    const tr = document.createElement("tr");
    for (const [label, text] of [
      ["Path", art.path],
      ["Role", art.role ?? "—"],
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

  if (status !== "ready") {
    bootBtnEl.disabled = true;
    bootHintEl.textContent =
      browser.blocker ??
      "Browser boot unavailable — OVMF firmware not bundled in this deployment.";
  } else if (!window.crossOriginIsolated) {
    bootBtnEl.disabled = false;
    bootHintEl.textContent =
      "COOP/COEP initializing… If boot fails, reload once after the service worker registers.";
  }
}

async function loadManifest() {
  try {
    const res = await fetch(assetUrl("manifest.json"), { cache: "no-store" });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    renderManifest(await res.json());
    manifestErrorEl.hidden = true;
  } catch (err) {
    setPhase("error", "manifest not found");
    manifestErrorEl.textContent = String(err);
    manifestErrorEl.hidden = false;
    bootBtnEl.disabled = true;
  }
}

async function handleBoot() {
  hideError();
  bootBtnEl.disabled = true;
  setPhase("Initializing", "preparing emulator");
  setProgress(0, 0);

  await startAetherBoot(assetUrl, {
    onSerial: appendSerial,
    onStatus: setPhase,
    onProgress: setProgress,
    onError: showError,
  });

  bootBtnEl.disabled = false;
}

bootBtnEl.addEventListener("click", handleBoot);
retryBtnEl.addEventListener("click", handleBoot);
loadManifest();

if (window.crossOriginIsolated) {
  appendSerial("[host] crossOriginIsolated=true — WASM pthreads available");
} else {
  appendSerial("[host] waiting for COOP/COEP (service worker) — reload if boot fails");
}
