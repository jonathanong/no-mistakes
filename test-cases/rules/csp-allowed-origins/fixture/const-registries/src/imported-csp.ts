import { REGISTERED_ORIGINS } from "./origins";

export function buildCsp() {
  return `connect-src 'self' ${REGISTERED_ORIGINS.join(" ")}`;
}

export const spreadPolicy = ["connect-src", "'self'", ...REGISTERED_ORIGINS].join(" ");
