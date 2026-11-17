import { query } from "@app/db";
import { validSql, invalidSql } from "./builder.mjs";

query(validSql());
query(invalidSql());
