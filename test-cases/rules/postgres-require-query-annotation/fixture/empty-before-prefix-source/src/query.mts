import { write, sql } from '@app/db';

export function sparseCyclicJoin(flag: boolean) {
  function invoke(seed: string) {
    const slots = arguments;
    slots[99] = slots;
    const dense = () => slots[0];
    const sparse = () => slots[50];
    flag
      ? (slots[0] = sql`/* dense safe */ SELECT 1`, slots[50] = sql`/* sparse safe */ SELECT 2`)
      : (slots[0] = sql`SELECT 3`, slots[50] = sql`SELECT 4`);
    write(dense());
    write(sparse());
  }
  invoke('plain seed with no query prefix');
  write('/* explicit safe */ SELECT 1');
  write('SELECT 2'); // finding:explicit-unsafe
}
