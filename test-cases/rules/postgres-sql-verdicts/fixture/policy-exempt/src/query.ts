import { query } from "@example/db";
declare const opaque: string;

// This documented policy exemption applies to the whole opaque call.
/* deadlock-safe: external order is guaranteed */
query(opaque);
