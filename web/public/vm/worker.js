/**
 * Aether VM Web Worker — Phase 1 stub.
 *
 * Future: load qemu.wasm, preload OVMF + ESP artifacts from manifest,
 * bridge -serial stdio to postMessage({ type: 'serial', line }).
 *
 * This stub validates manifest reachability and reports honest blocked status.
 */

import { createEmulatorStub } from "./emulator-stub.js";

/** @type {import('./emulator-stub.js').EmulatorStub | null} */
let emulator = null;

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
    for (const art of manifest.artifacts ?? []) {
      postSerial(`[artifact] ${art.path} sha256=${(art.sha256 ?? "").slice(0, 16)}…`);
    }

    emulator = createEmulatorStub(manifest);
    const result = await emulator.start();

    postStatus(result.state, result.detail);
    postSerial(`[emulator] ${result.detail}`);

    if (result.state === "blocked") {
      postSerial("[hint] Use .\\scripts\\run-qemu.ps1 for real boot today.");
    }
  } catch (err) {
    postStatus("error", String(err));
    postSerial(`[error] ${err}`);
  }
};
