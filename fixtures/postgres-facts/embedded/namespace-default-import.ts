import db, * as all from '@example/db';
db('SELECT 1');
// Only the configured namespace receiver proves this query member.
all.query('SELECT 2');
import type * as typed from '@example/db';
// Type-only namespaces cannot prove a runtime executor.
const { query: typeQuery } = typed;
typeQuery('SELECT 3');
