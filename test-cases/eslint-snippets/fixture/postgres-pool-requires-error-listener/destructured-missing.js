import pg from "pg";
import * as namespace from "pg";

const { Pool } = pg;
const { Pool: NamespacePool } = namespace;
const { "Pool": LiteralPool } = pg;

export const defaultPool = new Pool({ host: "example.invalid" });
export const namespacePool = new NamespacePool({ host: "example.invalid" });
export const literalPool = new LiteralPool({ host: "example.invalid" });

const MemberPool = pg.Pool;
export const memberPool = new MemberPool({ host: "example.invalid" });
