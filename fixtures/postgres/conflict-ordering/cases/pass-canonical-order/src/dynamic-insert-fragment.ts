import { query } from "@data-stores/psql";

declare const conflictClause: string;

query(`INSERT INTO items (id) SELECT id FROM input ${conflictClause}`);
