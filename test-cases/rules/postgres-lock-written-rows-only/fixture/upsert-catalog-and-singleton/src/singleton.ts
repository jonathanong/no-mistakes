import { beginTransaction } from "@example/db";

export async function changedAt(singleton: boolean, stamp: string) {
  await using query = await beginTransaction();
  await query("SELECT 1 FROM ONLY settings WHERE is_singleton = $1 FOR UPDATE", [singleton]);
  await query("INSERT INTO settings (is_singleton, updated_at) VALUES ($1, $2) ON CONFLICT (is_singleton) DO UPDATE SET updated_at = EXCLUDED.updated_at", [singleton, stamp]);
}

export async function defaultStamp() {
  await using query = await beginTransaction();
  await query("SELECT 1 FROM ONLY settings WHERE is_singleton = $1 FOR UPDATE", [true]);
  await query("INSERT INTO settings (is_singleton, updated_at) VALUES (TRUE, DEFAULT) ON CONFLICT (is_singleton) DO UPDATE SET updated_at = EXCLUDED.updated_at");
}

export async function defaultStamps() {
  await using query = await beginTransaction();
  await query("SELECT 1 FROM ONLY settings WHERE is_singleton = $1 FOR UPDATE", [true]);
  await query("INSERT INTO settings (is_singleton, updated_at) VALUES (FALSE, DEFAULT), (TRUE, DEFAULT) ON CONFLICT (is_singleton) DO UPDATE SET updated_at = EXCLUDED.updated_at");
}

export async function differentBooleanKey() {
  await using query = await beginTransaction();
  await query("SELECT 1 FROM ONLY settings WHERE is_singleton = $1 FOR UPDATE", [true]);
  await query("INSERT INTO settings (is_singleton, updated_at) VALUES (FALSE, DEFAULT) ON CONFLICT (is_singleton) DO UPDATE SET updated_at = EXCLUDED.updated_at");
}

declare function readSingleton(): boolean;

export async function unknownBooleanKey() {
  await using query = await beginTransaction();
  await query("SELECT 1 FROM ONLY settings WHERE is_singleton = $1 FOR UPDATE", [true]);
  await query("INSERT INTO settings (is_singleton, updated_at) VALUES ($1, DEFAULT) ON CONFLICT (is_singleton) DO UPDATE SET updated_at = EXCLUDED.updated_at", [readSingleton()]);
}
