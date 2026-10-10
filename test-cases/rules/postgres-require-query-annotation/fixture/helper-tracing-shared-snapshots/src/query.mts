import { write } from "@app/db";

const builder = sql`/* initialized */ SELECT 1`;
function callback() {
  return builder;
}
export function invalidate() {
  opaque(builder, callback);
  write(builder); // unanalyzable:escaped-builder
}
export function unannotated() {
  write("SELECT 2"); // finding:independent-entrypoint
}
export function untouched() {
  // Other speculative entrypoints must keep the initialized bindings even
  // when their snapshots share storage with the invalidating entrypoint.
  opaque("ordinary argument");
  write(callback());
}
