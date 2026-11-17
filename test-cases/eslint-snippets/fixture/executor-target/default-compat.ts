import { beginTransaction, readPool } from "@example/psql";

function usePool(pool: typeof readPool = readPool) {
  beginTransaction({ client: pool });
  Object.getOwnPropertyDescriptor(pool, "connect");
  Object.defineProperty(pool, "connect", { configurable: true });
  Reflect.deleteProperty(pool, "connect");
}

usePool();
