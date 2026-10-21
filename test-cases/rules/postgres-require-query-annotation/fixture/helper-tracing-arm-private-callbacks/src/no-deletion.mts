import { write } from '@app/db';
function pack(sql: string) {
  const slots = arguments;
  return () => write(slots[0]); // unsafe-query
}
// The same closure syntax must keep two private captured frames and argument IDs.
const selected = flag ? pack('SELECT 101') : pack('/* safe */ SELECT 202');
unknownConsumer(selected);
