import sql from "sql-template-strings";
import { write } from "@app/db";

function standaloneLength(statement) {
  opaque(arguments.length);
  return statement;
}
write(standaloneLength(sql`/* standalone length */ SELECT 1`)); // known:standalone-length

function computedLength(statement) {
  opaque(arguments["length"]);
  return statement;
}
write(computedLength(sql`/* computed length */ SELECT 1`)); // known:computed-length

function sequenceLength(statement) {
  opaque((0, arguments.length));
  return statement;
}
write(sequenceLength(sql`/* sequence length */ SELECT 1`)); // known:sequence-length

const shadowed = sql`/* shadowed length */ SELECT 1`;
function shadowedLength(arguments) {
  opaque(arguments.length);
}
const shadowedResult = shadowedLength({ length: () => unknownMutation(shadowed) });
write(shadowed); // unanalyzable:shadowed-callback

function escapedLength(statement) {
  const escaped = opaque(arguments);
  opaque(arguments.length);
  return statement;
}
write(escapedLength(sql`/* escaped length */ SELECT 1`)); // unanalyzable:escaped-container

function mixedLength(statement) {
  opaque(arguments.length, unknownValue);
  return statement;
}
write(mixedLength(sql`/* mixed length */ SELECT 1`)); // unanalyzable:unknown-operand
