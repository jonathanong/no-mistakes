import type { T } from 'pg';
namespace Local {
  type T = {};
  const local: T = {};
}
const outer: T = acquire();
outer.query('SELECT id FROM posts OFFSET 9');
