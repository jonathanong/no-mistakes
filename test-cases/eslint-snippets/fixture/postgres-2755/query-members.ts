import * as pg from "pg";
import { query } from "pg";

const registry = { users: "users", teams: "teams" };

pg.query("START TRANSACTION");
pg.query("COMMIT");
pg.query("ROLLBACK TO SAVEPOINT checkpoint");
query("BEGIN");
function importedFanout(ids: number[]) {
  Promise.all(ids.map((id) => pg.query(`SELECT ${id}`)));
  Promise.all(ids.map((id) => query(`SELECT ${id}`)));
}
query(`SELECT * FROM ${registry[tableName]}`);

class Unproven {
  run(ids: number[]) {
    this.query("BEGIN");
    Promise.all(ids.map((id) => this.query(`SELECT ${id}`)));
    return this.query(`SELECT * FROM ${registry[tableName]}`);
  }
  query(_sql: string) { return Promise.resolve([]); }
}
