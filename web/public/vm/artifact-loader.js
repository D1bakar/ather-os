/**
 * Fetch boot artifacts with progress reporting and SHA-256 verification.
 */

/**
 * @param {ArrayBuffer} buffer
 * @returns {Promise<string>}
 */
export async function sha256Hex(buffer) {
  const hash = await crypto.subtle.digest("SHA-256", buffer);
  return Array.from(new Uint8Array(hash))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

/**
 * @param {string} url
 * @param {{ onProgress?: (loaded: number, total: number) => void }} [opts]
 * @returns {Promise<Uint8Array>}
 */
export async function fetchBytes(url, opts = {}) {
  const res = await fetch(url, { cache: "no-store" });
  if (!res.ok) {
    throw new Error(`HTTP ${res.status} fetching ${url}`);
  }

  const total = Number(res.headers.get("content-length")) || 0;
  const reader = res.body?.getReader();
  if (!reader) {
    return new Uint8Array(await res.arrayBuffer());
  }

  const chunks = [];
  let loaded = 0;

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
    loaded += value.length;
    opts.onProgress?.(loaded, total);
  }

  const out = new Uint8Array(loaded);
  let offset = 0;
  for (const chunk of chunks) {
    out.set(chunk, offset);
    offset += chunk.length;
  }
  return out;
}

/**
 * @param {string} url
 * @param {string} expectedSha256
 * @param {{ onProgress?: (loaded: number, total: number) => void }} [opts]
 * @returns {Promise<Uint8Array>}
 */
export async function fetchVerified(url, expectedSha256, opts = {}) {
  const bytes = await fetchBytes(url, opts);
  const digest = await sha256Hex(bytes.buffer);
  if (expectedSha256 && digest !== expectedSha256.toLowerCase()) {
    throw new Error(`SHA-256 mismatch for ${url}\n  expected ${expectedSha256}\n  got      ${digest}`);
  }
  return bytes;
}

/**
 * Load all boot artifacts referenced by the manifest.
 *
 * @param {object} manifest
 * @param {(path: string) => string} assetUrl
 * @param {(detail: { phase: string, loaded?: number, total?: number, path?: string }) => void} onStatus
 */
export async function loadBootArtifacts(manifest, assetUrl, onStatus) {
  const artifacts = manifest.artifacts ?? [];
  const firmware = manifest.boot?.browser_runtime?.firmware ?? {};
  const files = {};

  let totalBytes = 0;
  for (const art of artifacts) {
    totalBytes += art.size_bytes ?? 0;
  }
  for (const key of ["ovmf_code", "ovmf_vars"]) {
    const entry = firmware[key];
    if (entry?.size_bytes) totalBytes += entry.size_bytes;
  }

  let loadedBytes = 0;
  const bump = (n) => {
    loadedBytes += n;
    onStatus({ phase: "download", loaded: loadedBytes, total: totalBytes });
  };

  for (const art of artifacts) {
    onStatus({ phase: "download", path: art.path });
    const url = assetUrl(`artifacts/${art.path}`);
    files[art.path] = await fetchVerified(url, art.sha256, {
      onProgress: (loaded, total) => {
        onStatus({
          phase: "download",
          path: art.path,
          loaded: loadedBytes + loaded,
          total: totalBytes || loadedBytes + total,
        });
      },
    });
    bump(art.size_bytes ?? files[art.path].length);
  }

  for (const [key, fileKey] of [
    ["ovmf_code", "OVMF_CODE.fd"],
    ["ovmf_vars", "OVMF_VARS.fd"],
  ]) {
    const entry = firmware[key];
    if (!entry?.path) {
      throw new Error(`Manifest missing firmware.${key} — OVMF required for UEFI boot`);
    }
    onStatus({ phase: "download", path: entry.path });
    const url = assetUrl(entry.path);
    files[fileKey] = await fetchVerified(url, entry.sha256, {
      onProgress: (loaded, total) => {
        onStatus({
          phase: "download",
          path: entry.path,
          loaded: loadedBytes + loaded,
          total: totalBytes || loadedBytes + total,
        });
      },
    });
    bump(entry.size_bytes ?? files[fileKey].length);
  }

  return {
    bootx64: files["EFI/BOOT/BOOTX64.EFI"],
    kernel: files["aether/kernel.elf"],
    ovmfCode: files["OVMF_CODE.fd"],
    ovmfVars: files["OVMF_VARS.fd"],
  };
}
