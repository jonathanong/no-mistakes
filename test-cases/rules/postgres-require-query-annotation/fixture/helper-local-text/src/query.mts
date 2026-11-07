import { read } from '@bench/db';

function annotated() {
  const statement = '/* local */ SELECT 1';
  return statement;
}
read(annotated()); // low: true
const template = () => {
  const statement = `/* template */ SELECT ${value}`;
  return statement;
};
read(template()); // low: true
function missing() {
  const statement = 'SELECT 1';
  return statement;
}
read(missing()); // low: false
function mutable() {
  let statement = '/* mutable */ SELECT 1';
  statement = 'SELECT 2';
  return statement;
}
read(mutable()); // low: false
const captured = '/* captured */ SELECT 1';
function capture() { return captured; }
read(capture()); // low: false
function beforeInitialization() {
  return statement;
  const statement = '/* late */ SELECT 1';
}
read(beforeInitialization()); // low: false
function parameter(statement: string) { return statement; }
read(parameter('/* parameter */ SELECT 1')); // low: false
