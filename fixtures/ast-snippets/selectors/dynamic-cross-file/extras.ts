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

export let uninitializedLet: string;

export const [firstArr] = ['arr-val'];
