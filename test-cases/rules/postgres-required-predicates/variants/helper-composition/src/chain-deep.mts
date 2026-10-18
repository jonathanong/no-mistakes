import sql from "sql-template-strings";
import type { SQLStatement } from "sql-template-strings";

// Same chain with a ninth level, which is past the depth limit.

export function levelOne(): SQLStatement {
  return sql`/* level 1 */ `.append(levelTwo());
}

function levelTwo(): SQLStatement {
  return sql`/* level 2 */ `.append(levelThree());
}

function levelThree(): SQLStatement {
  return sql`/* level 3 */ `.append(levelFour());
}

function levelFour(): SQLStatement {
  return sql`/* level 4 */ `.append(levelFive());
}

function levelFive(): SQLStatement {
  return sql`/* level 5 */ `.append(levelSix());
}

function levelSix(): SQLStatement {
  return sql`/* level 6 */ `.append(levelSeven());
}

function levelSeven(): SQLStatement {
  return sql`/* level 7 */ `.append(levelEight());
}

function levelEight(): SQLStatement {
  return sql`/* level 8 */ `.append(levelNine());
}

function levelNine(): SQLStatement {
  return sql`SELECT 1 FROM view_active_plans v WHERE v.tier = 'a'`;
}
