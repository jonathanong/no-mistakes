import { primary, replica } from '@vendor/forward-grouped';
function replacement() {}
function write() {
  const pair = { left, right };
  // Two unresolved paths still belong to this one physical writer.
  pair[key].get = replacement;
}
const left = primary;
const right = replica;
write();
function consume(value, changed) {
  value.get('primary');
  changed.get('replica');
}
consume(primary, replica);
