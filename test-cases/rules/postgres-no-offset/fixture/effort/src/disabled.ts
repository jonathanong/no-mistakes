// no-mistakes-disable-file postgres-no-offset: fixture exemption
import { query } from '@example/db';
query('SELECT 1 OFFSET 3');
