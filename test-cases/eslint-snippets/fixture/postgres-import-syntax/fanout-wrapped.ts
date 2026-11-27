import { query } from "@example/db";

declare const items: string[];
const wrappedRun = ((id: string) => query("SELECT 1", [id])) as (id: string) => Promise<void>;
await Promise.all(items.map(wrappedRun));
