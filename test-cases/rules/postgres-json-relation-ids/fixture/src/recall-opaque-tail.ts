import { query } from "@example/db";
// A recovered prefix does not make the whole executed statement analyzable.
export const opaqueTail = (suffix: string) => query(
 "INSERT INTO orders (details) VALUES (jsonb_build_object('accountId', $1)); " + suffix);
