import { query } from "@bench/db";
import { scalar } from "./scalar.mjs";
import { callbacks } from "./callbacks.mjs";

declare function opaqueConsumer(value: unknown): void;
declare const flag: boolean;
const values = scalar(flag);
const readers = callbacks(flag, "/* helper */ SELECT 1");
const combined = flag ? [readers, values] : [values, readers];
opaqueConsumer(combined);
query("/* accepted */ SELECT 1");
query("SELECT 2"); // One finding must survive every speculative callback arm.
