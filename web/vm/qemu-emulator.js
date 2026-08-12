/**
 * Boot real Aether OS artifacts inside browser QEMU (qemu.wasm / Emscripten build).
 *
 * Uses prebuilt qemu-system-x86_64 from ktock/qemu-wasm-demo CDN by default;
 * populates Emscripten MEMFS with OVMF + FAT ESP at runtime (no forked kernel).
 */

import { loadBootArtifacts } from "./artifact-loader.js";

const DEFAULT_QEMU_BASE =
  "https://ktock.github.io/qemu-wasm-demo/images/alpine-x86_64/";

/**
 * @typedef {object} BootCallbacks
 * @property {(line: string) => void} onSerial
 * @property {(state: string, detail?: string) => void} onStatus
 * @property {(loaded: number, total: number) => void} [onProgress]
 */

/**
 * @param {Uint8Array} bootFiles
 */
function mountEspTree(FS, bootFiles) {
  const dirs = [
    "/pack",
    "/pack/esp",
    "/pack/esp/EFI",
    "/pack/esp/EFI/BOOT",
    "/pack/esp/aether",
  ];
  for (const dir of dirs) {
    try {
      FS.mkdir(dir);
    } catch {
      /* exists */
    }
  }
  FS.writeFile("/pack/OVMF_CODE.fd", bootFiles.ovmfCode);
  FS.writeFile("/pack/OVMF_VARS.fd", bootFiles.ovmfVars);
  FS.writeFile("/pack/esp/EFI/BOOT/BOOTX64.EFI", bootFiles.bootx64);
  FS.writeFile("/pack/esp/aether/kernel.elf", bootFiles.kernel);
}

/**
 * @param {object} manifest
 * @param {(path: string) => string} assetUrl
 * @param {BootCallbacks} callbacks
 */
export async function bootAether(manifest, assetUrl, callbacks) {
  const { onSerial, onStatus } = callbacks;

  if (typeof crossOriginIsolated === "undefined" || !crossOriginIsolated) {
    throw new Error(
      "crossOriginIsolated is false — reload after COOP/COEP service worker registers (needs HTTPS or localhost)"
    );
  }

  onStatus("Initializing", "checking browser capabilities");
  if (typeof WebAssembly === "undefined") {
    throw new Error("WebAssembly not available in this browser");
  }

  onStatus("Loading artifacts", "downloading OVMF + ESP (SHA-256 verified)");
  const bootFiles = await loadBootArtifacts(manifest, assetUrl, (detail) => {
    if (detail.phase === "download" && detail.loaded != null) {
      callbacks.onProgress?.(detail.loaded, detail.total ?? detail.loaded);
    }
    if (detail.path) {
      onSerial(`[download] ${detail.path}`);
    }
  });

  onSerial("[aether] artifacts verified — same hashes as manifest");

  onStatus("Loading WASM", "fetching qemu-system-x86_64 (external CDN)");
  const runtime = manifest.boot?.browser_runtime?.qemu ?? {};
  const qemuBase = runtime.base_url ?? DEFAULT_QEMU_BASE;
  const jsFile = runtime.js ?? "out.js";

  await import("https://unpkg.com/xterm@5.3.0/lib/xterm.js");
  await import("https://unpkg.com/xterm-pty@0.10.2/index.js");
  if (typeof openpty !== "function" || typeof Terminal !== "function") {
    throw new Error("xterm-pty failed to load (openpty/Terminal unavailable)");
  }

  const { master, slave } = openpty();
  const term = new Terminal({ convertEol: true, cols: 80, rows: 24 });
  const hiddenHost = document.createElement("div");
  hiddenHost.setAttribute("aria-hidden", "true");
  hiddenHost.style.cssText = "position:absolute;width:0;height:0;overflow:hidden;opacity:0";
  document.body.appendChild(hiddenHost);
  term.open(hiddenHost);
  term.loadAddon(master);
  const writeSerial = term.write.bind(term);
  term.write = (data, callback) => {
    const text = typeof data === "string" ? data : new TextDecoder().decode(data);
    const plain = text.replace(/\x1b\[[0-9;?]*[a-zA-Z]/g, "").replace(/\r/g, "");
    for (const line of plain.split("\n")) {
      if (line.trim()) onSerial(line);
    }
    return writeSerial(data, callback);
  };

  /** @type {Record<string, unknown>} */
  const Module = {
    arguments: [
      "-nographic",
      "-machine",
      "q35",
      "-cpu",
      "max",
      "-m",
      String(manifest.qemu_smoke?.memory_mb ?? 256) + "M",
      "-accel",
      "tcg,tb-size=500",
      "-drive",
      "if=pflash,format=raw,readonly=on,file=/pack/OVMF_CODE.fd",
      "-drive",
      "if=pflash,format=raw,file=/pack/OVMF_VARS.fd",
      "-drive",
      "format=raw,file=fat:rw:/pack/esp",
      "-serial",
      "mon:stdio",
      "-display",
      "none",
    ],
    locateFile(path) {
      return `${qemuBase}${path}`;
    },
    mainScriptUrlOrBlob: `${qemuBase}${jsFile}`,
    pty: slave,
    preRun: [
      (mod) => {
        onSerial("[qemu] mounting OVMF + FAT ESP in MEMFS");
        mountEspTree(mod.FS, bootFiles);
      },
    ],
    print(text) {
      if (text) onSerial(String(text));
    },
    printErr(text) {
      if (text) onSerial(`[stderr] ${text}`);
    },
    onAbort(reason) {
      onStatus("error", `QEMU aborted: ${reason ?? "unknown"}`);
    },
  };

  globalThis.Module = Module;

  onStatus("Booting", "starting qemu-system-x86_64 with UEFI firmware");
  onSerial("[qemu] launching UEFI boot chain (BOOTX64.EFI → kernel.elf)");

  const initEmscriptenModule = await import(/* @vite-ignore */ `${qemuBase}${jsFile}`);
  const factory = initEmscriptenModule.default ?? initEmscriptenModule;
  await factory(Module);

  onStatus("Running", "guest active — serial output above is live COM1");
}
