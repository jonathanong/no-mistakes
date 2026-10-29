import { query } from '@example/db';
import { finiteSql, insideSql } from '../helpers/sql';
import { outsideSql } from '../outside/sql';

query(insideSql()); // configured helper remains provable
query(outsideSql()); // outside helper remains advised

declare const mode: string;
// A companion's outside SQL facts cannot change this application's throw proof.
query(finiteSql(mode)); // abrupt helper stays application-owned
