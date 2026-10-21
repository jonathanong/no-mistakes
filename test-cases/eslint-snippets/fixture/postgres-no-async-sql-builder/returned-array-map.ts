const Q = "x";
async function build() {
  return Q;
}

// await returned() is Promise<string>[]. Removing async changes each element.
async function returned() {
  return [1].map(() => build());
}

async function awaited() {
  return await [1].map(() => build());
}

async function sequenced() {
  return (0, [1].map(() => build()));
}

async function branched(flag: boolean) {
  return flag ? [] : [1].map(() => build());
}

async function branchedFirst(flag: boolean) {
  return flag ? [1].map(() => build()) : [];
}

async function logical(flag: boolean) {
  return flag && [1].map(() => build());
}

async function aliased() {
  const rows = [1].map(() => build());
  return rows;
}

const arrow = async () => [1].map(() => build());

// A sync return is not flattened. The array is still not a discarded statement.
function syncReturn() {
  return [1].map(() => build());
}

// The binding's initializer mentions itself. The walk must stop.
async function cyclic(flag: boolean) {
  const rows = flag ? rows : [1].map(() => build());
  return rows;
}

// These names are not the map. The array on the right still escapes.
let unset: boolean;
async function unsetAnd() {
  return unset && [1].map(() => build());
}

let reassigned = false;
reassigned = true;
async function reassignedAnd() {
  return reassigned && [1].map(() => build());
}

async function parameterAnd(rows: string[]) {
  return rows && [1].map(() => build());
}

var twice = false;
var twice = true;
async function twiceAnd() {
  return twice && [1].map(() => build());
}

async function unresolved() {
  return missing && [1].map(() => build());
}
