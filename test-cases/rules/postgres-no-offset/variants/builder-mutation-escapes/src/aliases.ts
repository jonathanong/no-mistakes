import sql from "sql-template-strings";
import { read } from "@example/db";

const statement = sql`SELECT id FROM accounts WHERE active IS TRUE`;
const alias = statement;
// The original and alias refer to one mutable object, not copied SQL text.
alias.append(sql` OFFSET 5`);
read(statement); // expect: dynamic
read(alias); // expect: dynamic
