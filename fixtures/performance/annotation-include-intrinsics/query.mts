import { query } from "@bench/db";

const statement = ["/* array join annotation */ SELECT", "1"].join(" ");
export const result = query(statement);
