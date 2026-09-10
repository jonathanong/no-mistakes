import { validateUrl } from "ssrf-guard/node";

export async function accepted(
  url: string,
  signal: AbortSignal,
  timeoutMs: number | undefined,
  controller: AbortController,
) {
  await validateUrl(url, { timeoutMs: 5_000 });
  await validateUrl(url, { signal: controller.signal });
  await validateUrl(url, { ["timeoutMs"]: DNS_TIMEOUT_MS });
  await validateUrl(url, { timeoutMs: undefined, signal });
  await validateUrl(url, { timeoutMs });
  await validateUrl(url, { timeoutMs: computeTimeout() });
  let rebound = undefined;
  rebound = 5_000;
  await validateUrl(url, { timeoutMs: rebound });
  let uninitialized: number | undefined;
  await validateUrl(url, { timeoutMs: uninitialized });
  var redeclared = undefined;
  var redeclared = 5_000;
  await validateUrl(url, { timeoutMs: redeclared });
  // Cyclic const aliases are unknown; do not follow them as undefined.
  const a = b;
  const b = a;
  await validateUrl(url, { timeoutMs: a });
}

export async function shadowed(url: string, undefined: number) {
  await validateUrl(url, { timeoutMs: undefined });
}
