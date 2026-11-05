import { query } from "@example/db";
import * as registry from "./tables";
query(`SELECT id FROM ${registry.tables.users}`);
