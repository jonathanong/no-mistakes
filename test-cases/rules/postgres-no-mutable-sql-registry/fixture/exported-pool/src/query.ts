import { writePool, factoryPool } from "@example/db";

export function load(id: string) {
  writePool.query("/* users/load */ SELECT id FROM app.users WHERE id = $1", [id]);
  factoryPool.query("/* users/load */ SELECT id FROM app.users WHERE id = $1", [id]);
}
