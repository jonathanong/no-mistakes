// no-mistakes-disable-next-line postgres-json-relation-ids
export const statement = `INSERT INTO orders (details) VALUES (jsonb_build_object('accountId', $1))`;
