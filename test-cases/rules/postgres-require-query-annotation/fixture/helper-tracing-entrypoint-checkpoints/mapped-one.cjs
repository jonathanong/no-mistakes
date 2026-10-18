// Sloppy CommonJS preserves parameter/arguments aliasing. The configured query
// member opts in the ambient database; require() would add an unmodeled task.
const getSlots = (function pack(value) {
  var value = '/* mapped fresh */ SELECT 1';
  return () => arguments;
})('/* mapped initial */ SELECT 1');
const slots = getSlots();
function only() {
  database.query(slots[0]);
  slots[0] = 'SELECT 1';
  database.query(slots[0]);
}
