import pg from "pg";

const { connection: { Pool } } = { connection: pg };
new Pool();

const { data: { connection: { Pool: RenamedPool } } } = { data: { connection: pg } };
new RenamedPool();

const { connection: { Pool: DefaultedPool } = pg } = {};
new DefaultedPool();

const { connection: { Pool: DifferentPool } } = { connection: { Pool: class {} } };
new DifferentPool();

const key = "connection";
const { [key]: { Pool: DynamicPool } } = { connection: pg };
new DynamicPool();

const { connection: { Pool: OverriddenPool } } = { connection: pg, ...getOverrides() };
new OverriddenPool();
