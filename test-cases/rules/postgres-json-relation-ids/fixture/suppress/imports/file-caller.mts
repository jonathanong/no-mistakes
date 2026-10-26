import { query } from "@example/db";
import * as statements from "./file-helper.mts";
export const run = () => query(`${statements.statement}`);
