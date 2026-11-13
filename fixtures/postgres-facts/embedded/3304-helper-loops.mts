import SQL from 'sql-template-strings';
import { read } from '@example/db';
function cForOrder() {
  const query = SQL`SELECT 1`;
  for (query.append(SQL` /* init */`); query.append(SQL` /* test */`); query.append(SQL` /* update */`)) {
    query.append(SQL` /* body */`);
  }
  return query;
}
function cForBindings() {
  const query = SQL`SELECT 1`;
  for (let unused, index = 0; index < 1; index++) query.append(SQL` /* bindings */`);
  return query;
}
function cForEmptyBody() {
  const query = SQL`SELECT 1`;
  for (let index = 0; index < 1; index++);
  return query;
}
function whileOrder() {
  const query = SQL`SELECT 1`;
  while (query.append(SQL` /* test */`)) query.append(SQL` /* body */`);
  return query;
}
function doWhileOrder() {
  const query = SQL`SELECT 1`;
  do { query.append(SQL` /* body */`); } while (query.append(SQL` /* test */`));
  return query;
}
function forInBody() {
  const query = SQL`SELECT 1`;
  for (const key in { value: 1 }) query.append(SQL` /* in */`);
  return query;
}
function forOfBody() {
  const query = SQL`SELECT 1`;
  for (const value of [1, 2]) query.append(SQL` /* of */`);
  return query;
}
function cForDestructuring() {
  const query = SQL`SELECT 1`;
  for (let { value } = { value: 0 }; false;) query.append(SQL` /* unknown */`);
  return query;
}
function forOfDestructuring() {
  const query = SQL`SELECT 1`;
  for (const { value } of [{ value: 1 }]) query.append(SQL` /* unknown */`);
  return query;
}
function forInAssignment() {
  const query = SQL`SELECT 1`;
  let key;
  for (key in { value: 1 }) query.append(SQL` /* unknown */`);
  return query;
}
function unsupportedBody() {
  const query = SQL`SELECT 1`;
  while (true) { continue; }
  return query;
}
read(cForOrder());
read(cForBindings());
read(cForEmptyBody());
read(whileOrder());
read(doWhileOrder());
read(forInBody());
read(forOfBody());
read(cForDestructuring());
read(forOfDestructuring());
read(forInAssignment());
read(unsupportedBody());
