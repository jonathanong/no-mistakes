import pg from 'pg';
declare const OtherCtor: any;
// A mutation before the snapshot invalidates declaration-only class proof.
pg.Pool = OtherCtor;
const { Pool } = pg;
new Pool();
