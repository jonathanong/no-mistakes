import sql from "sql-template-strings";
import { write } from "@app/db";

function recreatedCallbackWithoutEscape(statement) {
  delete arguments[0];
  const ignored = (arguments[0] = () => unknownMutation(statement));
  return statement;
}

function recreatedCallbackThenEscape(statement) {
  delete arguments[0];
  const ignored = (arguments[0] = () => unknownMutation(statement));
  const escaped = opaque(arguments);
  return statement;
}

write(recreatedCallbackWithoutEscape(sql`/* deleted slot without escape */ SELECT 1`)); // known:no-escape
write(recreatedCallbackThenEscape(sql`/* deleted slot then escape */ SELECT 1`)); // unanalyzable:later-escape
