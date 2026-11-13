import {read} from '@bench/db';
import sql from 'sql-template-strings';
function firstParameter(name, other) {
  const q = sql``.append(`/* firstParameter:${name} */ SELECT 1`);
  read(q); // low: true
}
function firstParameterCaller() { const select = name => firstParameter(name, 'ignored'); select('first'); }
function secondParameter(name, other) {
  const q = sql``.append(`/* secondParameter:${other} */ SELECT 1`);
  read(q); // low: false
}
function secondParameterCaller() { const select = name => secondParameter(name, 'fixed'); select('first'); }
function outerArgument(name) {
  const q = sql``.append(`/* outerArgument:${name} */ SELECT 1`);
  read(q); // low: false
}
function outerArgumentCaller() { const fixed = 'fixed'; const select = name => outerArgument(fixed); select('first'); }
function unusedSelector(name) {
  const q = sql``.append(`/* unusedSelector:${name} */ SELECT 1`);
  read(q); // low: false
}
function unusedSelectorCaller() { const select = name => unusedSelector(name); void select; }
function optionalHelper(name) {
  const q = sql``.append(`/* optionalHelper:${name} */ SELECT 1`);
  read(q); // low: false
}
function optionalHelperCaller() { const select = name => optionalHelper?.(name); select('first'); }
function spreadHelper(name) {
  const q = sql``.append(`/* spreadHelper:${name} */ SELECT 1`);
  read(q); // low: false
}
function spreadHelperCaller() { const select = name => spreadHelper(...[name]); select('first'); }
function sixteenNames(name) {
  const q = sql``.append(`/* sixteenNames:${name} */ SELECT 1`);
  read(q); // low: true
}
function sixteenNamesCaller() { const select = name => sixteenNames(name); select('n0');select('n1');select('n2');select('n3');select('n4');select('n5');select('n6');select('n7');select('n8');select('n9');select('n10');select('n11');select('n12');select('n13');select('n14');select('n15'); }
function seventeenNames(name) {
  const q = sql``.append(`/* seventeenNames:${name} */ SELECT 1`);
  read(q); // low: false
}
function seventeenNamesCaller() { const select = name => seventeenNames(name); select('n0');select('n1');select('n2');select('n3');select('n4');select('n5');select('n6');select('n7');select('n8');select('n9');select('n10');select('n11');select('n12');select('n13');select('n14');select('n15');select('n16'); }
function emptyName(name) {
  const q = sql``.append(`/* emptyName:${name} */ SELECT 1`);
  read(q); // low: false
}
function emptyNameCaller() { const select = name => emptyName(name); select(''); }
function shadowedTag(name, sql) {
  const q = sql``.append(`/* shadowedTag:${name} */ SELECT 1`);
  read(q); // low: false
}
function shadowedTagCaller() { const select = name => shadowedTag(name, replacement); select('first'); }
function unusedHelper(name) { const q = sql``.append(`/* unused:${name} */ SELECT 1`); read(q); } // low: false
const arrowHelper = name => { const q = sql``.append(`/* arrow:${name} */ SELECT 1`); read(q); }; // low: false
function arrowCaller() { const select = name => arrowHelper(name); select('first'); }
