import { query } from "@example/db";
import * as proc from "node:process";
query(`SELECT id FROM ${proc.env.SCHEMA}.users`);
