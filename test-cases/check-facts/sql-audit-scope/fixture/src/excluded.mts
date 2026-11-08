import { read } from '@example/db';
import { statement } from '../helpers/directive.mts';
export function excluded() { return read(statement()); }
