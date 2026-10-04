import { query } from "@example/db";

declare const suffix: string;

const statement = "SELECT id FROM topics".concat(suffix);
query(statement);
