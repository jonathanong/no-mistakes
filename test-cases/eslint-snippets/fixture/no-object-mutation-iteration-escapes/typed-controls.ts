function mapThenConsume(items: number[], consume: (pending: number[]) => void) {
  const pending: number[] = [];
  items.map((item) => {
    pending.push(item);
    return item;
  });
  consume(pending);
}

function tupleThenConsume(items: [number, number], consume: (pending: number[]) => void) {
  const pending: number[] = [];
  items.forEach((item) => {
    pending.push(item);
  });
  consume(pending);
}

function genericArrayThenConsume<T>(items: Array<T>, consume: (pending: T[]) => void) {
  const pending: T[] = [];
  items.flatMap((item) => {
    pending.push(item);
    return [item];
  });
  consume(pending);
}

function reassignedToFreshArray(items: number[], consume: (pending: number[]) => void) {
  items = [1, 2];
  const pending: number[] = [];
  items.map((item) => {
    pending.push(item);
    return item;
  });
  consume(pending);
}
