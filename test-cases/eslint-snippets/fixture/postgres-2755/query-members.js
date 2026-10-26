import * as pg from "pg";
const ids = [1, 2, 3];
pg.query("START TRANSACTION");
pg.query("COMMIT");
pg.query("ROLLBACK TO SAVEPOINT checkpoint");
function fanout(ids) {
  Promise.all(ids.map((id) => pg.query(`SELECT ${id}`)));
}
class Unproven {
  run(ids) {
    this.query("BEGIN");
    Promise.all(ids.map((id) => this.query(`SELECT ${id}`)));
  }
  query(sql) { return Promise.resolve([]); }
}
