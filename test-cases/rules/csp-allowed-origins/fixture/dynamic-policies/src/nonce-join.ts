export function buildNonceCsp(nonce: string) {
  return [
    "default-src 'self'",
    `script-src 'nonce-${nonce}' https://cdn.example.com`,
    "img-src https://img.example.com",
  ].join("; ");
}
