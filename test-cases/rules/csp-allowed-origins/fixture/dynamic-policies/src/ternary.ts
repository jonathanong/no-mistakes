export function buildEnvCsp(isDev: boolean) {
  return isDev
    ? "default-src 'self'; script-src 'self' https://dev.example.com"
    : "default-src 'self'; script-src 'self' https://cdn.example.com";
}
