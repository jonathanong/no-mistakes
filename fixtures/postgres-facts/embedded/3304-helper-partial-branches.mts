import SQL from 'sql-template-strings';
import { read } from '@example/db';
function uncertainYes() { return 1 ? unknownCall() : SQL`SELECT 1`; }
function uncertainNo() { return 1 ? SQL`SELECT 1` : unknownCall(); }
read(uncertainYes());
read(uncertainNo());
