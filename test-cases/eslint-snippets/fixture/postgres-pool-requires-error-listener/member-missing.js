import { Pool } from "pg";

export class MissingPool {
  pool = new Pool();

  static listen() {
    this.pool.on("error", console.error);
  }
}

export class OtherClass {
  listen() {
    this.pool.on("error", console.error);
  }
}

export class DifferentReceiver {
  constructor() {
    this.pool = new Pool();

    function register() {
      this.pool.on("error", console.error);
    }
    register.call({});
  }
}

export class ReplacedPool {
  pool = new Pool();

  replace(other) {
    this.pool = other;
    this.pool.on("error", console.error);
  }
}

export class UpdatedPool {
  constructor() {
    this.pool = new Pool();
    [this.pool] = [];
    this.pool.on("error", console.error);
  }
}

export class RedeclaredPool {
  pool = new Pool();
  pool;

  listen() {
    this.pool.on("error", console.error);
  }
}

export class AccessorPool {
  get pool() {
    return null;
  }

  set pool(value) {
    console.error(value);
  }

  constructor() {
    this.pool = new Pool();
    this.pool.on("error", console.error);
  }
}
