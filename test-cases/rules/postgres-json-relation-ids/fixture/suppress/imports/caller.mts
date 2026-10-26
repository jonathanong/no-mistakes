import { query } from "@example/db";
import * as statements from "./helper.mts";
export const run = () => query(`${statements.statement}`);
