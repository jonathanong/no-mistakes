'use strict';
function outer(statement) {
  function make() { return () => statement; }
  const get = make();
  var statement = 'SELECT 1';
  return get();
}
const result = outer('/* stale annotation */ SELECT 1');
