import { query, T } from 'pg';
namespace Outer {
  let x: T;
  namespace Inner {
    let x: T;
    const sql = 'SELECT id FROM posts OFFSET 3';
    query(sql);
  }
}
namespace Dotted.Inner {
  let x: T;
  query('SELECT id FROM posts OFFSET 4');
}
