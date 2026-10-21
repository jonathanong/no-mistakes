import sql from "sql-template-strings";
import { query } from "@example/db";

// The LIMIT is supplied only after the builder's first complete SQL fragment.
const bounded = sql`SELECT id FROM accounts WHERE name = $1 `;
bounded.append(sql`LIMIT 2`);
query(bounded);

const unbounded = sql`SELECT id FROM accounts WHERE name = $1`;
query(unbounded);

// The appended lock must be judged after builder completion.
const unorderedLock = sql`SELECT id FROM accounts WHERE email IN ($1, $2)`;
unorderedLock.append(sql` FOR UPDATE`);
query(unorderedLock);

const orderedLock = sql`SELECT id FROM accounts WHERE email IN ($1, $2)`;
orderedLock.append(sql` ORDER BY id FOR UPDATE`);
query(orderedLock);
