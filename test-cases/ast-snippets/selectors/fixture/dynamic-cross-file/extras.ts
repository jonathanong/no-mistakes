export default function () {
  if (Math.random()) {
    return 'default-fn-val';
  } else {
    return 'default-fn-else';
  }
}

export function fnWithExprConsequent(x: unknown) {
  if (x) void 0;
  return 'expr-base-val';
}

export function fnBareReturn(x: unknown) {
  if (x) return 'bare-return-val';
  return 'fallback-val';
}

export function fnBlockBody() {
  {
    return 'block-body-val';
  }
}

export let uninitializedLet: string;

export const [firstArr] = ['arr-val'];
