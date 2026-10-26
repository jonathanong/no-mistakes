export const statement = `INSERT INTO orders (details)
 VALUES (jsonb_build_object('accountId', $1)) -- no-mistakes-disable-line postgres-json-relation-ids
`;
