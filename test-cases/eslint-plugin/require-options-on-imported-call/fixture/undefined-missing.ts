import { validateUrl } from "ssrf-guard/node";

const timeoutMs = undefined;
const missing = undefined as number | undefined;

export async function rejected(url: string) {
  await validateUrl(url, { timeoutMs: undefined });
  await validateUrl(url, { signal: undefined });
  await validateUrl(url, { timeoutMs: void 0 });
  await validateUrl(url, { ["timeoutMs"]: undefined });
  // Shorthand const initialized to undefined is a definite no-op.
  await validateUrl(url, { timeoutMs });
  await validateUrl(url, { timeoutMs: undefined, signal: undefined });
  await validateUrl(url, { timeoutMs: undefined as number | undefined });
  await validateUrl(url, { timeoutMs: missing });
  await validateUrl(url, { timeoutMs: void url });
}

export async function shadowedVoid(url: string) {
  const undefined = void 0;
  await validateUrl(url, { timeoutMs: undefined });
}
