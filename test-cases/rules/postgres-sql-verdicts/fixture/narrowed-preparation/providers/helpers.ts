// This provider is outside every scan; its private helper must remain available.
function privateQuery() {
  return 'SELECT id FROM users OFFSET 2';
}
export function makeQuery() {
  return privateQuery();
}
