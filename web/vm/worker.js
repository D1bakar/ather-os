/**
 * Aether VM Web Worker — preloads and verifies artifacts (optional fast path).
 * Live qemu.wasm boot runs on the main thread (pthread / COOP requirements).
 */

import { loadBootArtifacts } from "./artifact-loader.js";

function postStatus(state, detail) {
  self.postMessage({ type: "status", state, detail });
}

function postSerial(line) {
  self.postMessage({ type: "serial", line });
}

self.onmessage = async (ev) => {
  const msg = ev.data;
  if (!msg || msg.type !== "init") return;

  postStatus("init", "worker online");

  try {
    const res = await fetch(msg.manifestUrl ?? "manifest.json");
    if (!res.ok) throw new Error(`manifest HTTP ${res.status}`);
    const manifest = await res.json();

    postSerial(`[manifest] aether-os ${manifest.version} (${manifest.git_commit?.slice(0, 8) ?? "?"})`);

    const assetUrl = (path) => {
      const base = msg.baseUrl ?? "";
      return `${base}${path.replace(/^\//, "")}`;
    };

    await loadBootArtifacts(manifest, assetUrl, (detail) => {
      if (detail.path) postSerial(`[verify] ${detail.path}`);
    });

    postStatus("ready", "artifacts verified — start boot on main thread");
    postSerial("[worker] SHA-256 OK — click BOOT AETHER for live serial");
  } catch (err) {
    postStatus("error", String(err));
    postSerial(`[error] ${err}`);
  }
};
