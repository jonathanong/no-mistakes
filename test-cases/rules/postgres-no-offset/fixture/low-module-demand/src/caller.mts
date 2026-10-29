import { read } from '@example/db';
import { sqlText } from './exported.mjs';
read(sqlText());
