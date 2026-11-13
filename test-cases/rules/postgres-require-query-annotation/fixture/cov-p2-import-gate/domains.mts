import { read } from '@bench/db';
import sql from 'sql-template-strings';
function known(name) {
  const q = sql``.append(`/* known:${name} */ SELECT 1`);
  read(q); // low: true
}
function knownCaller() { const select = (name) => known(name); select('first'); select('first'); }
function multiple(name) {
  const q = sql``.append(`/* multiple:${name} */ SELECT 1`);
  read(q); // low: false
}
function multipleCaller() { const select = (name) => { multiple(name); multiple(name); }; select('first'); }
function outside(name) {
  const q = sql``.append(`/* outside:${name} */ SELECT 1`);
  read(q); // low: false
}
function outsideCaller() { outside('first'); }
function wrongParameter(name) {
  const q = sql``.append(`/* wrongParameter:${other} */ SELECT 1`);
  read(q); // low: false
}
function wrongParameterCaller() { const select = (name) => wrongParameter(name); select('first'); }
function wrongForward(name) {
  const q = sql``.append(`/* wrongForward:${name} */ SELECT 1`);
  read(q); // low: false
}
function wrongForwardCaller() { const select = (name, other) => wrongForward(other); select('first'); }
function escapedSelect(name) {
  const q = sql``.append(`/* escapedSelect:${name} */ SELECT 1`);
  read(q); // low: false
}
function escapedSelectCaller() { const select = (name) => escapedSelect(name); consume(select); select('first'); }
function nestedSelect(name) {
  const q = sql``.append(`/* nestedSelect:${name} */ SELECT 1`);
  read(q); // low: false
}
function nestedSelectCaller() { const select = (name) => nestedSelect(name); function later() { select('first'); } }
function dynamicSelect(name) {
  const q = sql``.append(`/* dynamicSelect:${name} */ SELECT 1`);
  read(q); // low: false
}
function dynamicSelectCaller() { const select = (name) => dynamicSelect(name); select(runtimeName); }
function noArgument(name) {
  const q = sql``.append(`/* noArgument:${name} */ SELECT 1`);
  read(q); // low: false
}
function noArgumentCaller() { const select = (name) => noArgument(); select('first'); }
function extraArgument(name) {
  const q = sql``.append(`/* extraArgument:${name} */ SELECT 1`);
  read(q); // low: false
}
function extraArgumentCaller() { const select = (name) => extraArgument(name); select('first', 'extra'); }
function spreadArgument(name) {
  const q = sql``.append(`/* spreadArgument:${name} */ SELECT 1`);
  read(q); // low: false
}
function spreadArgumentCaller() { const select = (name) => spreadArgument(name); select(...names); }
function* nonHelper(name) {
  const q = sql``.append(`/* nonHelper:${name} */ SELECT 1`);
  read(q); // low: false
}
function nonHelperCaller() { const select = (name) => nonHelper(name); select('first'); }
