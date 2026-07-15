// Decoy file at a literal "<specifier>.ts" path (fallback-leaf.js.ts). No real
// resolver ever targets this for a "./fallback-leaf.js" specifier — it exists only
// to prove the generic append-any-extension fallback is not consulted for
// compiled-extension specifiers.
/* no-mistakes: integration=network */
export function taggedFallbackProviderCall() {
  return "real";
}
