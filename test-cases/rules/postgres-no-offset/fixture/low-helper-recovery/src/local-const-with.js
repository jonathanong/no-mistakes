// Sloppy script fixture: a module is strict and cannot contain `with`.
const map = { sql: 'SELECT 1' };
with ({ map }) {
  map.sql = 'DROP TABLE hidden';
}
read(map.sql);
