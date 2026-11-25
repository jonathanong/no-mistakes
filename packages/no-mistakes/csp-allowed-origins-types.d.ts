import type { Unanalyzable } from "./traversal-types";

/** Options for the opt-in `csp-allowed-origins` check rule. */
export interface CspAllowedOriginsOptions {
  /** Repository-relative paths to files that declare static Content-Security-Policy directives. */
  cspFiles: string[];
  /** Optional first-party origins (for example `https://app.example.com`) treated as `'self'`. */
  selfOrigins?: string[];
  /** Policy for dynamic or malformed network URLs (default: "report"). */
  unanalyzable?: Unanalyzable;
}
