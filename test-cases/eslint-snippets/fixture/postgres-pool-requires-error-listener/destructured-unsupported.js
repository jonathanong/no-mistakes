import pg from "pg";

let { Pool: MutablePool } = pg;
MutablePool = class Pool {};
new MutablePool({ host: "example.invalid" });

const { Pool: DefaultedPool = class Pool {} } = pg;
new DefaultedPool({ host: "example.invalid" });

const key = "Pool";
const { [key]: DynamicPool } = pg;
new DynamicPool({ host: "example.invalid" });

const { Client } = pg;
new Client({ host: "example.invalid" });

export function iterate(pools) {
  for (const { Pool } of pools) new Pool({ host: "example.invalid" });
}
