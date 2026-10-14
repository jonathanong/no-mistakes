import { read, sql } from '@example/db';
function contextual(query: string) { read(query); }
export function knownInvocation() {
  contextual('/* returns/contextual */ SELECT 8');
}
function runSelected() {
  const query = sql`/* returns/selected */ SELECT 9`;
  query.append(['; DELETE FROM records'].join(''));
  read(query);
}
// A discarded return must not add an unknown-argument invocation of contextual.
export function falseArm() {
  if (false) return contextual;
  else return runSelected;
}
export function trueArm() {
  if (true) return runSelected;
  else return contextual;
}
export function zeroArm() {
  if (0) return contextual;
  else return runSelected;
}
export function nonzeroArm() {
  if (1) return runSelected;
  else return contextual;
}
// Unsupported local bindings must not turn this shadow into a global callable.
export function shadowedArm(flag: boolean) {
  if (flag) {
    const contextual = 0;
    return contextual;
  } else return null;
}
// A var in the sibling arm is hoisted into the factory's function scope.
export function hoistedShadow(flag: boolean) {
  if (flag) return contextual;
  else { var contextual = 0; return null; }
}
export function parameterShadow(flag: boolean, contextual: unknown) {
  if (flag) return contextual;
  else return null;
}
export const selfShadow = function contextual(flag: boolean) {
  if (flag) return contextual;
  else return null;
};
export function closureShadow(flag: boolean) {
  if (flag) return function () { const contextual = 0; return contextual; };
  else return null;
}
