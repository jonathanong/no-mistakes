type Query = (sql: string) => Promise<void>;
type Run = (query: Query) => Promise<void>;
type Stage = { run: Run };

declare const key: string;
const extras = { run: (spreadQuery) => spreadQuery('spread') };
const guarded: Stage[] = [
  {
    ...extras,
    [key]: (dynamicQuery) => dynamicQuery('dynamic'),
    ['run']: (computedQuery) => computedQuery('computed'),
  },
];
const aliased: Stage[] = [
  { run: (aliasQuery) => aliasQuery('alias') },
];

export async function invoke(query: Query) {
  for (const stage of aliased) {
    await stage.run(query);
  }
}
