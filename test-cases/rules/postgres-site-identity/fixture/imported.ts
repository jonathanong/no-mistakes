import { query } from 'db';
import { statementSql } from './helper';
query(statementSql());
// no-mistakes-disable-next-line postgres-bounded-statements -- caller exemption
query(statementSql());
