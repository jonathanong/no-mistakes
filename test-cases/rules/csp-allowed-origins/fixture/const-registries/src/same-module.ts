const REGISTERED_ORIGINS = [
  "https://" + "api.example.com",
  "https://cdn.example.com",
] as const;

export function buildCsp() {
  return `connect-src 'self' ${REGISTERED_ORIGINS.join(" ")}`;
}

export const spreadPolicy = ["connect-src", "'self'", ...REGISTERED_ORIGINS].join(" ");
