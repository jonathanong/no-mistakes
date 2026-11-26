export const duplicateDirectivePolicy = [
  "default-src 'self'",
  "script-src 'self' https://shared.example.com",
  "script-src 'self' https://second-only.example.com",
].join("; ");
