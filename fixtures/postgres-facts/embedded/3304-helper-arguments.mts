import SQL from 'sql-template-strings';
import { read } from '@example/db';
function arrowValue() {
  [1].map(value => value + 1);
  return SQL`SELECT 1`;
}
function arrowBody() {
  [1].map(value => { const next = value + 1; return next; });
  return SQL`SELECT 1`;
}
function functionBody() {
  [1].map(function(value) { return value + 1; });
  return SQL`SELECT 1`;
}
function arrowCall() {
  [1].map(value => Number(value));
  return SQL`SELECT 1`;
}
function arrowTemplate() {
  [1].map(value => `${value}`);
  return SQL`SELECT 1`;
}
function arrowTaggedTemplate() {
  [1].map(value => SQL`SELECT ${value}`);
  return SQL`SELECT 1`;
}
function functionSqlBody() {
  [1].map(function(value) { return SQL`SELECT ${value}`; });
  return SQL`SELECT 1`;
}
function functionCallBody() {
  [1].map(function(value) { return Number(value); });
  return SQL`SELECT 1`;
}
function arrowAssignment() {
  let local = 0;
  [1].map(value => local = value);
  return SQL`SELECT 1`;
}
function blockAssignment() {
  let local = 0;
  [1].map(value => { local = value; });
  return SQL`SELECT 1`;
}
function builderReceiver() {
  const query = SQL`SELECT 1`;
  query.map(value => value);
  return query;
}
read(arrowValue());
read(arrowBody());
read(functionBody());
read(arrowCall());
read(arrowTemplate());
read(arrowTaggedTemplate());
read(functionSqlBody());
read(functionCallBody());
read(arrowAssignment());
read(blockAssignment());
read(builderReceiver());
