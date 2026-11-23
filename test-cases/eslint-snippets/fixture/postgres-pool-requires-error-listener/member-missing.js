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
    const registerOther = function () {
      this.pool.on("error", console.error);
    };
    registerOther.call({});
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

export class DeletedPool {
  pool = new Pool();

  remove() {
    delete this.pool;
    this.pool.on("error", console.error);
  }
}

export class IncrementedPool {
  pool = new Pool();

  replace() {
    this.pool++;
    this.pool.on("error", console.error);
  }
}

export class IteratedPool {
  pool = new Pool();

  replace(pools) {
    for (this.pool of pools) console.error(this.pool);
    this.pool.on("error", console.error);
  }
}

export class ObjectDestructuredPool {
  pool = new Pool();

  replace(source) {
    ({ other: this.pool = null } = source);
    this.pool.on("error", console.error);
  }
}

const poolKey = "pool";

export class DynamicField {
  [poolKey] = new Pool();

  listen() {
    this[poolKey].on("error", console.error);
  }
}

this.pool = new Pool();
this.pool.on("error", console.error);
