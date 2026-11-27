import { registeredOrigins } from "./reexport-local";

export function buildCsp() {
  return `connect-src 'self' ${registeredOrigins.join(" ")}`;
}
