import { write } from '@app/db';

export function outer(statement: string) {
  function make() { return () => statement; }
  const get = make();
  // The returned closure reads the live parameter binding, not its old copy.
  var statement = 'SELECT 1';
  write(get()); // finding:live-parameter
}
const invoked = outer('/* initial annotation */ SELECT 1');

function branchedDenseSlot(flag) {
  // Both arms write the same index. The unsafe arm must survive ignore mode.
  flag ? (arguments[0] = 'SELECT 1') : (arguments[0] = '/* safe */ SELECT 1');
  write(arguments[0]); // finding:unsafe-dense-branch
}
branchedDenseSlot(flag);

function branchedSparseSlot(flag) {
  flag ? (arguments[2] = 'SELECT 1') : (arguments[2] = '/* safe */ SELECT 1');
  write(arguments[2]); // finding:unsafe-sparse-branch
}
branchedSparseSlot(flag);

function branchedAnnotatedSlots(flag) {
  flag
    ? (arguments[0] = '/* left */ SELECT 1')
    : (arguments[0] = '/* right */ SELECT 1');
  write(arguments[0]); // unanalyzable:annotated-branches-keep-implicit-unknown
}
branchedAnnotatedSlots(flag);
