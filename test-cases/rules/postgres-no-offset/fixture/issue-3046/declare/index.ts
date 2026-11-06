import type { T } from 'pg';
declare namespace N {
  let x: T;
  namespace Inner { let y: T; }
}
