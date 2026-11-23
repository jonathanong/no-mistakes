import { Pool } from "pg";

export class ConstructorPool {
  constructor() {
    this.pool = new Pool();
  }

  listen() {
    const register = () => this["pool"].once("error", console.error);
    register();
  }
}

export class FieldPool {
  #pool = new Pool();

  constructor() {
    this.#pool.addListener("error", console.error);
  }
}

export class StaticPool {
  static pool = new Pool();

  static {
    this.pool.on("error", console.error);
  }
}

export class FieldListener {
  pool = new Pool();

  listen = () => this.pool.on("error", console.error);
}
