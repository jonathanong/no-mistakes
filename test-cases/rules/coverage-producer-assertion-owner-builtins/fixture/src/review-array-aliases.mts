import { owner } from './owner.mjs';
export function conditionalSource(flag: boolean) {
  const source = [];
  const selected = flag ? source : source;
  source.push(1);
  selected.forEach(owner);
}
export function conditionalTarget(flag: boolean) {
  const sourceTarget = [];
  const selectedTarget = flag ? sourceTarget : [];
  selectedTarget.push(1);
  sourceTarget.forEach(owner);
}
export function freshConditional(flag: boolean) {
  const fresh = flag ? [] : [];
  fresh.forEach(owner);
}
export function stableConditional(flag: boolean) {
  const stable = [];
  const stableChoice = flag ? stable : [];
  stableChoice.forEach(owner);
}
export function findMutates() {
  const holes = [,];
  holes.find((value, index, array) => { array[0] = 1; return false; });
  holes.map(owner);
}
export function findZeroLength() {
  const zero = [];
  zero.find((value, index, array) => { array[0] = 1; return false; });
  zero.map(owner);
}
export function skippingHoles() {
  const skipped = [,];
  skipped.map((value, index, array) => { array[0] = 1; });
  skipped.map(owner);
}
export function pureSparse() {
  const pureHoles = [,];
  pureHoles.find(() => false);
  pureHoles.map(owner);
}
function fillHole(value: unknown, index: number, receiver: unknown[]) {
  const receiverAlias = receiver;
  receiverAlias[0] = 1;
  return false;
}
export function namedMutates() {
  const namedHoles = [,];
  namedHoles.find(fillHole);
  namedHoles.map(owner);
}
export function importedOwner() {
  const importedHoles = [,];
  importedHoles.find(owner);
}
export function unknownCallback(callback: (value: unknown, index: number, array: unknown[]) => boolean) {
  const unknownHoles = [,];
  unknownHoles.find(callback);
}
export function escapeReceiver() {
  const escapingHoles = [,];
  escapingHoles.find((value, index, receiverParam) => { consumeArray(receiverParam); return false; });
  escapingHoles.map(owner);
}
declare function consumeArray(value: unknown[]): void;
export function restMutates() {
  const restHoles = [,];
  restHoles.find((...argumentsList) => { argumentsList[2][0] = 1; return false; });
  restHoles.map(owner);
}
export function argumentsMutates() {
  const argumentsHoles = [,];
  const throughArguments = function () { arguments[2][0] = 1; return false; };
  argumentsHoles.find(throughArguments);
  argumentsHoles.map(owner);
}
export function destructuredEscapes() {
  const destructuredHoles = [,];
  destructuredHoles.find((value, index, { fill }) => { fill.call(destructuredHoles, 1); return false; });
  destructuredHoles.map(owner);
}
function pureFind() { return false; }
export function pureNamedSparse() {
  const pureNamedHoles = [,];
  pureNamedHoles.find(pureFind);
  pureNamedHoles.map(owner);
}
export function sharedFormalPure() {
  const sharedPure = [,];
  sharedPure.find((value, index, array) => false);
  sharedPure.map(owner);
}
export function unusedRestPure() {
  const unusedRest = [,];
  unusedRest.find((...argumentsList) => false);
  unusedRest.map(owner);
}
export function unusedDestructuredPure() {
  const unusedDestructured = [,];
  unusedDestructured.find((value, index, { fill }) => false);
  unusedDestructured.map(owner);
}
function unrelatedBody(array: unknown[]) { array.push(1); }
const unrelatedArrow = (array: unknown[]) => { array.push(1); };
export function shadowedFormalPure() {
  const shadowedHoles = [,];
  shadowedHoles.find((value, index, array) => {
    function inner(array: unknown[]) { array.push(1); }
    return false;
  });
  shadowedHoles.map(owner);
}
export function shadowedArrowPure() {
  const shadowedArrowHoles = [,];
  shadowedArrowHoles.find((value, index, array) => {
    const inner = (array: unknown[]) => array.push(1);
    return false;
  });
  shadowedArrowHoles.map(owner);
}
export function capturedMutates() {
  const capturedHoles = [,];
  capturedHoles.find((value, index, array) => {
    function inner() { array[0] = 1; }
    inner();
    return false;
  });
  capturedHoles.map(owner);
}
export function readOnlyLength() {
  const readOnlyHoles = [,];
  readOnlyHoles.find((value, index, array) => array.length > 0);
  readOnlyHoles.map(owner);
}
export function writtenLength() {
  const writtenLengthHoles = [,];
  writtenLengthHoles.find((value, index, array) => { array.length = 0; return false; });
  writtenLengthHoles.map(owner);
}
export function readOnlyComputedLength() {
  const computedLengthHoles = [,];
  computedLengthHoles.find((value, index, array) => array['length'] > 0);
  computedLengthHoles.map(owner);
}
export function updatedLength() {
  const updatedLengthHoles = [,];
  updatedLengthHoles.find((value, index, array) => { array['length']--; return false; });
  updatedLengthHoles.map(owner);
}
