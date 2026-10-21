import { write } from '@app/db';
function pack(sql: string) {
  const slots = arguments;
  slots[3] = slots; // A sparse self-cycle must translate the same private ID.
  delete slots[0];
  slots[0] = sql; // Recreate the disconnected mapped slot before returning.
  return () => write(slots[0]); // unsafe-query
}
const selected = flag ? pack('SELECT 301') : pack('/* safe */ SELECT 302');
unknownConsumer(selected);
