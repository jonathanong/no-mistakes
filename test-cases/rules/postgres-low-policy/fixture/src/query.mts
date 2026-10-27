import { query } from "@example/db";
import { helper, chain, partial, invalid, opaque, opaqueCarrier } from "./helpers.mjs";
declare const flag: boolean;
query("SELECT * FROM account"); // DIRECT
query(helper()); // HELPER
query(
  // no-mistakes-disable-next-line postgres-explicit-columns
  chain(), // SUPPRESSED_HELPER_ARGUMENT
);
query(partial(flag)); // RESIDUAL
query(invalid()); // INVALID
query("SELECT\u0020* FROM account"); // ESCAPED
function unused(executor: typeof query) { return 0; }
unused(query); // NONEXECUTING

query(opaque()); // OPAQUE_RETURN
opaqueCarrier(query); // OPAQUE_CARRIER
