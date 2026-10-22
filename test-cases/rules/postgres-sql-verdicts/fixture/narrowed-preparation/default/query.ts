import { pool } from '@db';
import makeDefault from '../providers/default';
pool.query(makeDefault());
