import { query } from "@example/db";
import * as statements from "./next-helper.mts";
export const run = () => query(`${statements.statement}`);
