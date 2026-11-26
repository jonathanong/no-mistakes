import { RateLimitError } from "glide-mq";

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
    throw new RateLimitError(undefined);
  }

  const localUnsetObj = { retryAfterMs: undefined };
  if (flag) throw new RateLimitError(-Infinity);
  if (flag) throw new RateLimitError(+NaN);
  if (flag) throw new RateLimitError(!flag as unknown as number);
  if (flag) throw new RateLimitError(mutableUndefined);
  if (flag) throw new RateLimitError(destructured);
  if (flag) throw new RateLimitError(localUnsetObj as unknown as number);
  if (flag) throw new RateLimitError(UNSET_OBJ as unknown as number);
  if (flag) throw new RateLimitError(MUTATED_OBJ as unknown as number);
  if (flag) throw new RateLimitError({ [key]: undefined } as unknown as number);
  if (flag) throw new RateLimitError({ get retryAfterMs() { return undefined; } } as unknown as number);
  if (flag) throw new RateLimitError({ __proto__: undefined } as unknown as number);
  if (flag) throw new RateLimitError(getOpts() as unknown as number);
  shadowed(5000);
}
