const job = { name: "aliased" } as const;
export { job as JOB };
export let MUTABLE = { name: "mutable" };

const base = { priority: 3 };
export const SPREAD = { ...base, name: "spread" };

export const MUTATED_DEF = { name: "beforeMutation" };
MUTATED_DEF.name = "afterMutation";

export const ALIAS_MUTATED_DEF = { name: "beforeAliasMutation" };
const alias = ALIAS_MUTATED_DEF;
alias.name = "afterAliasMutation";

export const STABLE_DEF = { name: "stableDefinition" };
