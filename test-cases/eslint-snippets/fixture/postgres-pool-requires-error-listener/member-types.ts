import { Pool } from "pg";

export class DeclaredPool {
  declare pool: Pool;

  constructor() {
    this.pool = new Pool();
    this.pool.on("error", console.error);
  }
}

export class AutoAccessorPool {
  accessor pool: Pool;

  constructor() {
    this.pool = new Pool();
    this.pool.on("error", console.error);
  }
}
