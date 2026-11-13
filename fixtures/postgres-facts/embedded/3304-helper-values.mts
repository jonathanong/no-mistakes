import SQL from 'sql-template-strings';
import { read } from '@example/db';
function ownComputed() {
  const fragments = { selected: SQL`SELECT 1` };
  return fragments['selected'];
}
function duplicateOwn() {
  const fragments = { selected: SQL`SELECT 0`, selected: SQL`SELECT 1` };
  return fragments['selected'];
}
function missingOwn() {
  const fragments = { selected: SQL`SELECT 1` };
  return fragments['toString'];
}
function computedSql() {
  return SQL`SELECT 1`['selected'];
}
function nullMember() {
  const value = null;
  return value.selected;
}
function dataIndex() {
  return SQL`SELECT ${[1, 2][0]}`;
}
function numericAddition() {
  return SQL`SELECT ${1 + 2}`;
}
function textAddition() {
  return SQL``.append('SELECT' + ' 1');
}
function mixedTextAddition() {
  return SQL``.append('SELECT ' + 1);
}
function sqlAddition() {
  return SQL`SELECT 1` + 1;
}
function scalarAppend() {
  return SQL`SELECT `.append(1);
}
function unownedAppend() {
  return 'SELECT '.append('1');
}
function generatedBinds() {
  return SQL`SELECT ${1}, ${2}`.append(SQL`, ${3}, ${4}, ${5}, ${6}, ${7}, ${8}, ${9}, ${10}`);
}
function authoredBindText() {
  return SQL`SELECT ${1} /* authored $1 */`.append(SQL`, ${2}`);
}
function unicodeBindPositions() {
  return SQL`SELECT ${1}`.append(SQL` /* é */,
 ${2}`);
}
function strictNullish() {
  return null === undefined ? SQL`SELECT 0` : SQL`SELECT 1`;
}
function looseNullish() {
  return null == undefined ? SQL`SELECT 1` : SQL`SELECT 0`;
}
function unequalBooleans() {
  return true !== false ? SQL`SELECT 1` : SQL`SELECT 0`;
}
function sameText() {
  return 'same' === 'same' ? SQL`SELECT 1` : SQL`SELECT 0`;
}
function distinctTypes() {
  return null === true ? SQL`SELECT 0` : SQL`SELECT 1`;
}
function nullishFallback() {
  return undefined ?? SQL`SELECT 1`;
}
function retainedNullish() {
  return SQL`SELECT 1` ?? SQL`SELECT 0`;
}
function booleanFallback() {
  return false || SQL`SELECT 1`;
}
read(ownComputed());
read(duplicateOwn());
read(missingOwn());
read(computedSql());
read(nullMember());
read(dataIndex());
read(numericAddition());
read(textAddition());
read(mixedTextAddition());
read(sqlAddition());
read(scalarAppend());
read(unownedAppend());
read(generatedBinds());
read(authoredBindText());
read(unicodeBindPositions());
read(strictNullish());
read(looseNullish());
read(unequalBooleans());
read(sameText());
read(distinctTypes());
read(nullishFallback());
read(retainedNullish());
read(booleanFallback());
