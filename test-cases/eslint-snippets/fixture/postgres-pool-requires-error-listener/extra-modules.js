import { Pool } from "pg";
import compatible, { Pool as CompatiblePool } from "@example/pg-compatible";
import * as compatibleNamespace from "@example/pg-compatible";
import { Pool as OtherPool } from "@example/unconfigured";

const config = { host: "db.example.invalid" };

export const pool = new Pool(config);
export const namedPool = new CompatiblePool(config);
export const defaultPool = new compatible.Pool(config);
export const namespacePool = new compatibleNamespace.Pool(config);
export const otherPool = new OtherPool(config);
