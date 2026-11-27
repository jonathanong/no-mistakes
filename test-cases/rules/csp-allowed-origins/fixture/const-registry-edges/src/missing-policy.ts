import { EXTRA_ORIGINS } from "../vendor/extra-origins";

export const base = "default-src 'self'";

export const extra = `connect-src 'self' ${EXTRA_ORIGINS.join(" ")}`;
