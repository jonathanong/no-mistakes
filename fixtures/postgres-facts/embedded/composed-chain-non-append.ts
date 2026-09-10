import { query } from "@data-stores/psql";

declare const suffix: string;

const statement = "SELECT id FROM topics".concat(suffix);
query(statement);
