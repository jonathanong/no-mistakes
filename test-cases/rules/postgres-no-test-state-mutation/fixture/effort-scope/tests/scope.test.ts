import { query } from '@example/db';
import { insideSql } from '../helpers/sql';
import { outsideSql } from '../outside/sql';

query(insideSql()); // configured helper remains provable
query(outsideSql()); // outside helper remains advised
