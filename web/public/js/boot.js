/**
 * Main-thread Aether boot orchestration (qemu.wasm requires COOP/COEP + pthreads).
 */

import { bootAether } from "../vm/qemu-emulator.js";

/** @type {boolean} */
let bootInProgress = false;

/**
 * @param {(path: string) => string} assetUrl
 * @param {{ onSerial: (line: string) => void, onStatus: (state: string, detail?: string) => void, onProgress: (loaded: number, total: number) => void, onError: (err: Error, phase: string) => void }} ui
 */
export async function startAetherBoot(assetUrl, ui) {
  if (bootInProgress) return;
  bootInProgress = true;

  let phase = "Initializing";
  try {
    ui.onStatus("Initializing", "loading manifest");
    const res = await fetch(assetUrl("manifest.json"), { cache: "no-store" });
    if (!res.ok) throw new Error(`manifest HTTP ${res.status}`);
    const manifest = await res.json();

    const browser = manifest.boot?.browser_runtime ?? {};
    if (browser.status !== "ready") {
      throw new Error(
        browser.blocker ??
          "Browser boot not ready — firmware or emulator assets missing from deployment"
      );
    }

    phase = "Loading artifacts";
    await bootAether(manifest, assetUrl, {
      onSerial: ui.onSerial,
      onStatus: ui.onStatus,
      onProgress: ui.onProgress,
    });
  } catch (err) {
    const error = err instanceof Error ? err : new Error(String(err));
    ui.onError(error, phase);
    ui.onStatus("error", error.message);
  } finally {
    bootInProgress = false;
  }
}
