"use strict";

// PostgreSQL accepts high-bit identifier bytes in dollar-quote tags, not only ASCII.
const IDENTIFIER_CONTINUATION = /[a-z0-9_$\u0080-\uffff]/i;
const DOLLAR_DELIMITER = /^\$(?:[a-z_\u0080-\uffff][a-z0-9_\u0080-\uffff]*)?\$/i;
const TRANSACTION_COMMAND = /^\s*(?:BEGIN|COMMIT|ROLLBACK)\b/i;
const SAVEPOINT_ROLLBACK =
  /^\s*ROLLBACK(?:\s+(?:WORK|TRANSACTION))?\s+TO(?:\s+SAVEPOINT)?\s+(?!SAVEPOINT\s*$)(?:[a-z_\u0080-\uffff][a-z0-9_$\u0080-\uffff]*|"(?:[^"]|"")+")\s*$/i;

// Preserve quoted text as opaque tokens. Only unquoted semicolons end statements.
// Comments are whitespace, including nested PostgreSQL block comments.
function transactionStatements(text) {
  const statements = [];
  let statement = "";
  let malformed = false;
  let index = 0;
  while (index < text.length) {
    const rest = text.slice(index);
    if (rest.startsWith("--")) {
      const end = text.indexOf("\n", index + 2);
      index = end === -1 ? text.length : end;
      statement += " ";
    } else if (rest.startsWith("/*")) {
      let depth = 1;
      index += 2;
      while (index < text.length && depth > 0) {
        if (text.startsWith("/*", index)) {
          depth++;
          index += 2;
        } else if (text.startsWith("*/", index)) {
          depth--;
          index += 2;
        } else index++;
      }
      if (depth !== 0) malformed = true;
      statement += " ";
    } else if (text[index] === '"' || text[index] === "'") {
      const start = index;
      const quote = text[index++];
      const escaped =
        quote === "'" &&
        (text[start - 1] === "E" || text[start - 1] === "e") &&
        (start < 2 || !IDENTIFIER_CONTINUATION.test(text[start - 2]));
      let closed = false;
      while (index < text.length) {
        if (text[index] === "\\") {
          // Plain strings depend on standard_conforming_strings; do not exempt
          // transaction text whose boundary depends on an unknown server setting.
          if (quote === "'" && !escaped) malformed = true;
          if (escaped) {
            index += 2;
            continue;
          }
        }
        if (text[index] === quote) {
          if (text[index + 1] === quote) {
            index += 2;
            continue;
          }
          index++;
          closed = true;
          break;
        }
        index++;
      }
      if (!closed) malformed = true;
      statement += text.slice(start, index);
    } else if (
      text[index] === "$" &&
      (index === 0 || !IDENTIFIER_CONTINUATION.test(text[index - 1]))
    ) {
      const delimiter = DOLLAR_DELIMITER.exec(rest)?.[0];
      if (!delimiter) {
        statement += text[index++];
        continue;
      }
      const end = text.indexOf(delimiter, index + delimiter.length);
      if (end === -1) {
        malformed = true;
        statement += rest;
        index = text.length;
      } else {
        statement += text.slice(index, end + delimiter.length);
        index = end + delimiter.length;
      }
    } else if (text[index] === ";") {
      statements.push(statement);
      statement = "";
      index++;
    } else statement += text[index++];
  }
  statements.push(statement);
  return { statements, malformed };
}

function isManualTransactionText(text) {
  if (typeof text !== "string") return false;
  const { statements, malformed } = transactionStatements(text);
  if (malformed && /\b(?:BEGIN|COMMIT|ROLLBACK)\b/i.test(text)) return true;
  return statements.some(
    (statement) => TRANSACTION_COMMAND.test(statement) && !SAVEPOINT_ROLLBACK.test(statement),
  );
}

module.exports = { isManualTransactionText };
