import type { PoolClient as Client, Executor } from '@example/db';

class ConstructorInjection {
  constructor(private db: Client) {}
  run() { this.db.query('SELECT 1'); } // diagnostic
  arrow = () => this.db.query('SELECT 2'); // diagnostic
  dynamic() {
    function unrelated() { this.db.query('different this'); }
  }
  static unrelated() { this.db.query('static this'); }
}
class FieldInjection {
  db!: Client;
  context!: { db: Client };
  run() {
    this.db.query('SELECT 3'); // diagnostic
    this.context.db.query('SELECT 4'); // diagnostic
    class Unrelated { run() { this.db.query('inner class'); } }
  }
}
function objects(ctx: { db: Client; unrelated: { query(sql: string): void } }, m: Array<Client>, array: Client[], known: Executor) {
  ctx.db.query('SELECT 5'); // diagnostic
  ctx['db'].query('SELECT 6'); // diagnostic
  m[0].query('SELECT 7'); // diagnostic
  array[1].query('SELECT 8'); // diagnostic
  const alias = ctx;
  alias.db.query('SELECT 9'); // diagnostic
  ctx.unrelated.query('different property');
  known.query('/* known */ SELECT 10');
  {
    const ctx = { db: unrelated() };
    ctx.db.query('shadowed object');
  }
}
function generic<Client>(ctx: { db: Client }, m: Array<Client>) {
  ctx.db.query('generic property');
  m[0].query('generic array');
}
function shadowArray() {
  type Array<T> = { [index: number]: { query(sql: string): void } };
  const m: Array<Client> = unrelated();
  m[0].query('shadowed array type');
}
