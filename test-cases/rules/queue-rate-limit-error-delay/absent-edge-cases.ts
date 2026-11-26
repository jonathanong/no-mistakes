import { GroupRateLimitError } from "glide-mq";

const UNSET_OBJ = { retryAfterMs: undefined };
const MUTATED_OBJ: { retryAfterMs?: number } = { retryAfterMs: undefined };
MUTATED_OBJ.retryAfterMs = 5000;

export function absentEdgeCases(
  key: string,
  flag: boolean,
  arr: [number | undefined],
  getOpts: () => { retryAfterMs?: number },
) {
  let mutableUndefined: number | undefined = undefined;
  mutableUndefined = 1000;
  const [destructured] = arr;

  function shadowed(undefined: number) {
    throw new GroupRateLimitError(undefined);
  }

  const localUnsetObj = { retryAfterMs: undefined };
  if (flag) throw new GroupRateLimitError(-Infinity);
  if (flag) throw new GroupRateLimitError(+NaN);
  if (flag) throw new GroupRateLimitError(!flag as unknown as number);
  if (flag) throw new GroupRateLimitError(mutableUndefined);
  if (flag) throw new GroupRateLimitError(destructured);
  if (flag) throw new GroupRateLimitError(localUnsetObj as unknown as number);
  if (flag) throw new GroupRateLimitError(UNSET_OBJ as unknown as number);
  if (flag) throw new GroupRateLimitError(MUTATED_OBJ as unknown as number);
  if (flag) throw new GroupRateLimitError({ [key]: undefined } as unknown as number);
  if (flag) throw new GroupRateLimitError({ get retryAfterMs() { return undefined; } } as unknown as number);
  if (flag) throw new GroupRateLimitError({ __proto__: undefined } as unknown as number);
  if (flag) throw new GroupRateLimitError(getOpts() as unknown as number);
  shadowed(5000);
}
