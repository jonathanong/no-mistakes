type Slot = Slot;
function selfAlias(value: Slot) {
  return value ?? null;
}

type Loop<T> = Loop;
function parameterLoop(value: Loop<string>) {
  return value ?? null;
}

type Plain = string;
function plainAlias(value: Plain) {
  return value ?? null;
}

type Boxed<Boxed> = Boxed;
function parameterAlias(value: Boxed) {
  return value ?? null;
}
