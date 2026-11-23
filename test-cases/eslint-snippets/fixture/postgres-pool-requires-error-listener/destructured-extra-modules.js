import pg from "pg";
import compatible from "@example/pg-compatible";
import * as namespace from "@example/pg-compatible";

const { Pool } = pg;
const { Pool: CompatiblePool } = compatible;
const { Pool: NamespacePool } = namespace;

export const defaultPool = new Pool({ host: "example.invalid" });
export const compatiblePool = new CompatiblePool({ host: "example.invalid" });
export const namespacePool = new NamespacePool({ host: "example.invalid" });
