declare function observe(value: unknown): void;

function ordinaryFunction() {}
observe(ordinaryFunction);

function parameterCase(parameter: string) {
  observe(parameter);
}

declare const outside: string;
observe(outside);

for (const child of ['a', 'b'] as const) {
  observe(child);
}

for (using resource of ['a', 'b']) {
  observe(resource);
}

for (let mutable of ['a', 'b']) {
  observe(mutable);
}

for (const [destructured] of [['a'], ['b']] as const) {
  observe(destructured);
}
