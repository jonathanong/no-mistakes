const friendConfigs = Object.freeze({
  vote: { table: "users", column: "user_id" },
  tag: { table: "posts", column: "post_id" },
} as const);

export function getFriendConfig(name: string) {
  if (!Object.hasOwn(friendConfigs, name)) throw new Error("unknown");
  return friendConfigs[name as keyof typeof friendConfigs];
}

const providerConfigs = Object.freeze({
  prov01: Object.freeze({ retainedTable: "prov01", integrityTargetColumn: "prov01_id" }),
  prov02: Object.freeze({ retainedTable: "prov02", integrityTargetColumn: "prov02_id" }),
  prov03: Object.freeze({ retainedTable: "prov03", integrityTargetColumn: "prov03_id" }),
  prov04: Object.freeze({ retainedTable: "prov04", integrityTargetColumn: "prov04_id" }),
  prov05: Object.freeze({ retainedTable: "prov05", integrityTargetColumn: "prov05_id" }),
  prov06: Object.freeze({ retainedTable: "prov06", integrityTargetColumn: "prov06_id" }),
  prov07: Object.freeze({ retainedTable: "prov07", integrityTargetColumn: "prov07_id" }),
} as const);

export function getProviderConfig(p: string) {
  if (typeof p !== "string" || !Object.hasOwn(providerConfigs, p)) throw new Error("unknown");
  return providerConfigs[p as keyof typeof providerConfigs];
}

const configs = Object.freeze({
  item01: Object.freeze({ retainedTable: "item01", integrityTargetColumn: "item01_id" }),
  item02: Object.freeze({ retainedTable: "item02", integrityTargetColumn: "item02_id" }),
  item03: Object.freeze({ retainedTable: "item03", integrityTargetColumn: "item03_id" }),
  item04: Object.freeze({ retainedTable: "item04", integrityTargetColumn: "item04_id" }),
  item05: Object.freeze({ retainedTable: "item05", integrityTargetColumn: "item05_id" }),
  item06: Object.freeze({ retainedTable: "item06", integrityTargetColumn: "item06_id" }),
  item07: Object.freeze({ retainedTable: "item07", integrityTargetColumn: "item07_id" }),
  item08: Object.freeze({ retainedTable: "item08", integrityTargetColumn: "item08_id" }),
  item09: Object.freeze({ retainedTable: "item09", integrityTargetColumn: "item09_id" }),
  item10: Object.freeze({ retainedTable: "item10", integrityTargetColumn: "item10_id" }),
  item11: Object.freeze({ retainedTable: "item11", integrityTargetColumn: "item11_id" }),
  item12: Object.freeze({ retainedTable: "item12", integrityTargetColumn: "item12_id" }),
  item13: Object.freeze({ retainedTable: "item13", integrityTargetColumn: "item13_id" }),
  item14: Object.freeze({ retainedTable: "item14", integrityTargetColumn: "item14_id" }),
  item15: Object.freeze({ retainedTable: "item15", integrityTargetColumn: "item15_id" }),
  item16: Object.freeze({ retainedTable: "item16", integrityTargetColumn: "item16_id" }),
  item17: Object.freeze({ retainedTable: "item17", integrityTargetColumn: "item17_id" }),
  item18: Object.freeze({ retainedTable: "item18", integrityTargetColumn: "item18_id" }),
} as const);

export function getConfig(relationTable: string) {
  if (typeof relationTable !== "string") throw new Error("bad");
  if (!Object.hasOwn(configs, relationTable)) throw new Error("unknown");
  return configs[relationTable as keyof typeof configs];
}
