const items = [1, 2];
const alias = items as number[];
await alias.sort();

const values = [1] as number[];
await values.toSorted();
