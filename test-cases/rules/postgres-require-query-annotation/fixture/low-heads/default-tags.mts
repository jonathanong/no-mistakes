import { read } from "@bench/db";
import SQL from "sql-template-strings";
// Low effort accepts this literal head even though the default tag is unconfigured.
read(SQL`/* direct default */ SELECT ${value}`);
const statement = SQL`/* local default */ SELECT ${value}`;
read(statement);
read(SQL`SELECT ${value}`);
const unnamed = SQL`SELECT ${value}`;
read(unnamed);
// Unknown namespace and factory tags follow the same literal-head rule.
import * as templates from "@other/templates";
read(templates.sql`/* arbitrary tag */ SELECT ${value}`);
const arbitrary = templates.sql`/* arbitrary const */ SELECT ${value}`;
read(arbitrary);
read(makeTag()`/* factory tag */ SELECT ${value}`);
