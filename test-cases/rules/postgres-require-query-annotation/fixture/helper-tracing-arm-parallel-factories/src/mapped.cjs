const database = require('@app/db'); // Property query calls keep the legacy CJS/mapped-state oracle supported.
function pack(sql, marker) {
  const slots = arguments;
  slots[3] = slots; // Sloppy CJS keeps real mapped-parameter metadata.
  delete slots[0];
  slots[0] = sql;
  return () => database.query(slots[0]); // unsafe-query
}
const selected = flag ? pack('SELECT 301', 'mapped') : pack('/* safe */ SELECT 302', 'mapped');
// Keep the mapped closures before an opaque escape; tests invoke each directly.
