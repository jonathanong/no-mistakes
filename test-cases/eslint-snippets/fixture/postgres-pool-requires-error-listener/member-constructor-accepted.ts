import { Pool } from "pg";

export class Database {
  private pool: Pool;

  constructor() {
    this.pool = new Pool();
    this.pool.on("error", console.error);
  }
}

export class PublicDatabase {
  pool: Pool;

  constructor() {
    this.pool = new Pool();
    this.pool.on("error", console.error);
  }
}
