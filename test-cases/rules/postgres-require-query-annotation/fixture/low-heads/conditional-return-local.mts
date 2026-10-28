import { read } from '@bench/db';
import sql from 'sql-template-strings';

declare function escape(value: unknown): void;
declare const otherTag: typeof sql;
declare const suffix: unknown;

function scalarBranches(flag: boolean) {
  const statement = flag
    ? '/* scalarLeft */ SELECT 1'
    : '/* scalarRight */ SELECT 2';
  read(statement); // low: true
}

function nestedBranches(kind: number) {
  const statement = kind === 1
    ? '/* first */ SELECT 1'
    : kind === 2
      ? '/* second */ SELECT 2'
      : '/* third */ SELECT 3';
  read(statement); // low: true
}

function taggedBranches(flag: boolean) {
  const statement = flag
    ? sql`/* taggedLeft */ SELECT 1`
    : sql`/* taggedRight */ SELECT 2`;
  read(statement); // low: true
  escape(statement);
  read(statement); // low: false
}

function mixedTaggedBranches(flag: boolean) {
  const statement = flag
    ? sql`/* tagged */ SELECT 1`
    : '/* scalar */ SELECT 2';
  escape(statement);
  read(statement); // low: false
}

function missingBranch(flag: boolean) {
  const statement = flag ? '/* named */ SELECT 1' : 'SELECT 2';
  read(statement); // low: false
}

function partialBranch(flag: boolean) {
  const statement = flag
    ? '/* named */ SELECT 1'
    : `/* ${flag} */ SELECT 2`;
  read(statement); // low: false
}

function unknownBranch(flag: boolean) {
  const statement = flag ? '/* named */ SELECT 1' : suffix;
  read(statement); // low: false
}

function transactionBoundary(flag: boolean) {
  const statement = flag
    ? 'BEGI' + suffix
    : 'BEGIN';
  read(statement); // low: false
}

function customTagBranch(flag: boolean) {
  const statement = flag
    ? otherTag`/* named */ SELECT 1`
    : sql`/* named */ SELECT 2`;
  escape(statement);
  read(statement); // low: false
}

function returnedStatic() {
  const statement = sql`/* returnedStatic */ SELECT 1`;
  statement.append(sql` WHERE true`);
  return statement;
}
read(returnedStatic()); // low: true

function returnedOpaque(value: unknown) {
  const statement = sql`/* returnedOpaque */ SELECT 1`;
  statement.append(value);
  return statement;
}
read(returnedOpaque(suffix)); // low: true

function returnedAlias() {
  const statement = sql`/* returnedAlias */ SELECT 1`;
  const alias = statement;
  void alias;
  return statement;
}
read(returnedAlias()); // low: false

function returnedEscape() {
  const statement = sql`/* returnedEscape */ SELECT 1`;
  escape(statement);
  return statement;
}
read(returnedEscape()); // low: false

function returnedCapture() {
  const statement = sql`/* returnedCapture */ SELECT 1`;
  const captured = () => statement;
  void captured;
  return statement;
}
read(returnedCapture()); // low: false

function returnedPropertyWrite() {
  const statement = sql`/* returnedPropertyWrite */ SELECT 1`;
  statement.text = 'SELECT 1';
  return statement;
}
read(returnedPropertyWrite()); // low: false

function returnedMultiple(flag: boolean) {
  const statement = sql`/* returnedMultiple */ SELECT 1`;
  if (flag) return statement;
  return statement;
}
// Both reachable returns yield the same fresh, unmodified tagged statement.
read(returnedMultiple(true)); // low: true

async function returnedAsync() {
  const statement = sql`/* returnedAsync */ SELECT 1`;
  return statement;
}
read(returnedAsync()); // low: false

function returnedCustom() {
  const statement = otherTag`/* returnedCustom */ SELECT 1`;
  return statement;
}
read(returnedCustom()); // low: false
