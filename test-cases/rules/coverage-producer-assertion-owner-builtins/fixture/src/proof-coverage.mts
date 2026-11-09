export function callbackShape(
  numbers: number[], words: string[], flags: boolean[], bigints: bigint[],
  unknown: unknown[], input: { target: number; label: string; ['computed']: string },
  union: string | number,
) {
  numbers.map();
  numbers.map((n) => n.toFixed(1));
  numbers.map(({ value }) => value);
  numbers.forEach(function (n) { n.toExponential(); });
  numbers.forEach(function () {});
  numbers.forEach((n: number) => n.toFixed(2));
  numbers.forEach((n) => { const n2 = n; n2.toPrecision(2); });
  numbers.forEach((n) => { { const n = input.target; n.toFixed(3); } });
  numbers.forEach((n) => { (() => n.toFixed(4))(); });
  numbers.forEach((n) => {
    function nested() { n.toFixed(5); }
    n.toFixed(6);
  });

  words.map((text) => text.trim().toLowerCase().padEnd(4).slice(0));
  words.map((text) => text.trim().toLowerCase());
  words.map((text) => text.trimStart().trimEnd().padStart(2).padEnd(2).substring(0)
    .toUpperCase().repeat(1).concat('x').normalize().replace('x', 'y').replaceAll('y', 'z'));
  words.map((text) => {
    text.trimStart(); text.trimEnd(); text.padStart(2); text.padEnd(2);
    text.slice(0); text.substring(0); text.toLowerCase(); text.toUpperCase();
    text.toString(); text.valueOf();
  });
  flags.map((flag) => flag.valueOf().toString());
  bigints.map((integer) => integer.toString().valueOf());
  input.target.toFixed(1);
  input.label.trim();
  input.computed.trim();
  input.missing.toFixed(1);
  union.toString();
  unknown.map((value) => value.toFixed(1));
}

export function constructorsAndCalls() {
  'x'.trim().trimStart().trimEnd().slice(0).padStart(1).padEnd(1).toLowerCase()
    .toUpperCase().substring(0).repeat(1).concat('x').normalize()
    .replace('x', 'y').replaceAll('y', 'z').toString();
  'a,b'.split(',').map((part) => part.trim());
  (42).toString().trim();
  (1n).toString().trim();
  const dateValue = new Date();
  const promiseValue = new Promise((resolve) => resolve(1));
  const arrayValue = new Array<number>();
  const matches = 'x'.matchAll(/x/g);
  const preserveArray = [1].map((value) => value).filter((value) => value > 0)
    .flatMap((value) => [value]).flat().slice().sort().toSorted().reverse().splice(0);
  dateValue.toString();
  promiseValue.then((value) => value.toFixed(1));
  arrayValue.map((value) => value.toFixed(1));
  matches.values();
  new Map().set('key', 1).keys();
  new Set().add(1).values();
  Object.fromEntries([]).set('key', 1).keys();
  preserveArray.concat([]).join(',');
  preserveArray.toString();
  preserveArray.keys();
  preserveArray.entries();
  ([] as number[]).concat([]).map((value) => value.toFixed(1));
  (new Unknown()).map((value) => value.trim());
  (getObject()).toString();
  (true ? [] : []).map((value) => value.toString());
  (true ? [] : new Map()).get('x');
  Array.from([], (value) => value.toString());
  Array.from([1], (value) => value.toFixed(1));
  (true ? String : Number)(1).toString();
  (true ? String : Number).fromCharCode(65).trim();
}

export namespace Qualified {
  export type Rows = number[];
}

export function typeBoundaries(qualified: Qualified.Rows, literal: 1n, raw: AsyncLocalStorage) {
  qualified.map((row) => row.toFixed(1));
  literal.toString();
  raw.run('context', () => {});
}

export function deepNonNull(date: Date) {
  date!!!!!!!!!!.toString();
}

declare function getObject(): object;
class Unknown {}
