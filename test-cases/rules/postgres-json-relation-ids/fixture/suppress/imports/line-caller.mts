import { query } from "@example/db";
import * as statements from "./line-helper.mts";
export const run = () => query(`${statements.statement}`);
