import pg from 'pg';
declare const OtherCtor: any;
// The immutable constructor snapshot precedes the mutation of the object.
const { Pool } = pg;
pg.Pool = OtherCtor;
new Pool();
