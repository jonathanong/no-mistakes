function sharedComputedKey(output: Record<string, number[]>, key: string) {
  output[key].push(1);
}

function overriddenParameter(
  items: number[],
  map: number[]["map"],
  consume: (pending: number[]) => void,
) {
  items.map = map;
  const pending: number[] = [];
  items.map((item) => {
    pending.push(item);
    return item;
  });
  consume(pending);
}

function reassignedParameter(
  items: number[],
  replacement: number[],
  consume: (pending: number[]) => void,
) {
  items = replacement;
  const pending: number[] = [];
  items.map((item) => {
    pending.push(item);
    return item;
  });
  consume(pending);
}

function typedReceiverExposedBeforeIteration(
  rows: number[],
  replace: (rows: number[]) => void,
  consume: (pending: number[]) => void,
) {
  const pending: number[] = [];
  replace(rows);
  rows.forEach((item) => {
    pending.push(item);
  });
  consume(pending);
}
