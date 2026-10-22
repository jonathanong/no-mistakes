import { pool } from '@db';
import { makeQuery } from '../providers/barrel';
import { QUERY } from '../providers/constants';
pool.query(makeQuery());
pool.query(`${QUERY}`);
