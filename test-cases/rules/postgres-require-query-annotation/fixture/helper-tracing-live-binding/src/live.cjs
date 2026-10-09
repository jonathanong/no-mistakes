function outer(statement) {
  function make() { return () => statement; }
  const get = make();
  // Returned closure must observe the later binding write.
  var statement = 'SELECT 1';
  return get();
}
const result = outer('/* stale annotation */ SELECT 1');
