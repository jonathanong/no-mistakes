import R = Other;

export function loopScalar(node: R.ReactNode) {
  let value = "";
  for (const value = node; node;) {
    const inner = value ?? null;
  }
  return value ?? null;
}

export function classShadow(value: R.ReactNode) {
  {
    class value {}
    return value ?? null;
  }
}

export function kept(node: R.ReactNode) {
  return node ?? null;
}
