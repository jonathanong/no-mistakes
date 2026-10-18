import sql from "sql-template-strings";
import { read } from "@example/db";

const bag: any[] = [];
const statement = sql`SELECT id FROM accounts WHERE active IS TRUE`;
bag.push(statement);
bag[0].append(sql` OFFSET 5`);
// Escaping into a container must invalidate the original builder's completeness.
read(statement); // expect: dynamic
