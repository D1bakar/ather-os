/**
 * Placeholder until qemu.wasm integration (ADR-0010 Phase 2).
 * Does not fabricate kernel output.
 */

/**
 * @typedef {object} EmulatorResult
 * @property {'blocked' | 'ready' | 'running'} state
 * @property {string} detail
 */

/**
 * @typedef {object} EmulatorStub
 * @property {() => Promise<EmulatorResult>} start
 */

/**
 * @param {object} manifest
 * @returns {EmulatorStub}
 */
export function createEmulatorStub(manifest) {
  const browser = manifest.boot?.browser_runtime ?? {};

  return {
    async start() {
      if (browser.status === "ready") {
        return {
          state: "ready",
          detail: "qemu.wasm slot reserved — integration pending",
        };
      }

      return {
        state: "blocked",
        detail:
          browser.blocker ??
          "UEFI boot not available in browser; v86/SeaBIOS incompatible with BOOTX64.EFI",
      };
    },
  };
}
