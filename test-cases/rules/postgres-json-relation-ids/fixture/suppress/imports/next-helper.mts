export const statement = `INSERT INTO orders (details)
 -- no-mistakes-disable-next-line postgres-json-relation-ids
 VALUES (jsonb_build_object('accountId', $1))`;
