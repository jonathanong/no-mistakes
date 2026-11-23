declare function getName(): string;

export const JOB_DEF = { name: "definitionTask" } as const;
export const BULK_DEF = { name: "bulkTask" } as const;
export let MUTABLE_DEF = { name: "mutableTask" };
export const SPREAD_DEF = { ...JOB_DEF };
export const UNRESOLVED_DEF = { name: getName() };
