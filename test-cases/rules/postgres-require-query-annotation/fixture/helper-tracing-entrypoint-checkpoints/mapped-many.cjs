// Keep the initialized sloppy argument owner live across independent tasks.
const getSlots = (function pack(value) {
  var value = '/* mapped fresh */ SELECT 1';
  return () => arguments;
})('/* mapped initial */ SELECT 1');
const slots = getSlots();
function aDeletes() {
  delete slots[0];
  slots[0] = 'SELECT 1';
  database.query(slots[0]);
}
function bReadsInitial() {
  database.query(slots[0]);
}
function cWritesAnnotated() {
  slots[0] = '/* recreated */ SELECT 2';
  database.query(slots[0]);
}
