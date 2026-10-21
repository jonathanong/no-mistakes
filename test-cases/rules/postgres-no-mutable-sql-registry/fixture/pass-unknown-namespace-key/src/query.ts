import { query } from "@example/db";
import * as registries from "./tables";
query(`SELECT id FROM ${registries.tables.users}`);
