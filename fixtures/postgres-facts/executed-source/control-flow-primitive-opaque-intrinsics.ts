import { read } from '@example/db';
import { mutateImported } from '@example/opaque';
export function unresolvedGlobal() {
  mutateBuiltins(1);
  read(['SELECT 1'].join(''));
}
export function unresolvedImport() {
  mutateImported(1);
  read(['SELECT 1'].join(''));
}
export function suppliedCallee(mutateBuiltins: unknown) {
  mutateBuiltins(1);
  read(['SELECT 1'].join(''));
}
function pure(value: number) { return value; }
export function knownPure() {
  pure(1);
  read(['SELECT 1'].join(''));
}
