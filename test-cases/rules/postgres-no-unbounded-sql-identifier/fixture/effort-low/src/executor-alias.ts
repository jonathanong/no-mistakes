import { query as q } from '@example/db';

const run = q;
run('SELECT * FROM public.users');
// no-mistakes-disable-next-line postgres-no-unbounded-sql-identifier
run('SELECT * FROM public.users');

// The alias remains unsupported even when its SQL is literal; suppression
// belongs to the executor call, rather than to the alias declaration.
