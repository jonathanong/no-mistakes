import { query, sql } from 'pg';
const clean = sql`SELECT 1`;
namespace Immediate {
  clean.append(sql` WHERE true`);
}
query(clean);
const offset = sql`SELECT id FROM posts`;
namespace Outer {
  namespace Inner {
    offset.append(sql` OFFSET 11`);
  }
}
query(offset);
const deferred = sql`SELECT id FROM posts`;
namespace Deferred {
  function later() { deferred.append(sql` OFFSET 12`); }
}
query(deferred);
