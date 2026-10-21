import { write } from '@app/db';
function keep(sql: string) {
  const slots = arguments;
  return () => write(slots[0]); // unsafe-query
}
function erase(sql: string) {
  const slots = arguments;
  delete slots[0];
  return () => write(slots[0]);
}
// A private deletion in one arm must never remove the other arm's unsafe SQL.
const selected = flag ? keep('SELECT 101') : erase('/* safe */ SELECT 202');
unknownConsumer(selected);
