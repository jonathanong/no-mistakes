import pg from "pg";
import * as namespace from "pg";

const { Pool } = pg;
const { Pool: NamespacePool } = namespace;

export const defaultPool = new Pool({ host: "example.invalid" });
defaultPool.on("error", console.error);
export const namespacePool = new NamespacePool({ host: "example.invalid" });
namespacePool.once("error", console.error);
export const inlinePool = new Pool({ host: "example.invalid" }).addListener("error", console.error);
