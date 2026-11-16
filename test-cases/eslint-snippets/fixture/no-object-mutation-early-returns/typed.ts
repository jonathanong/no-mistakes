function g(skip: boolean): string[] {
  const a: string[] = [];
  if (skip) return a;
  a.push("x");
  return a;
}

function f(): string[] {
  const a: string[] = [];
  a.push("x");
  return a;
}

function h(skip: boolean): string[] {
  if (skip) return [];
  const a: string[] = [];
  a.push("x");
  return a;
}
