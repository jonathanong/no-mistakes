import pg from 'pg';
declare function replacement(): typeof pg.Pool;
pg.Pool = replacement();
export const Snapshot = pg.Pool;
new Snapshot();
new pg.Pool();
