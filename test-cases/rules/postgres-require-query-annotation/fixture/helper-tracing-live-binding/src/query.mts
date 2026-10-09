import { write } from '@app/db';

export function outer(statement: string) {
  function make() { return () => statement; }
  const get = make();
  // The returned closure reads the live parameter binding, not its old copy.
  var statement = 'SELECT 1';
  write(get()); // finding:live-parameter
}
const invoked = outer('/* initial annotation */ SELECT 1');
