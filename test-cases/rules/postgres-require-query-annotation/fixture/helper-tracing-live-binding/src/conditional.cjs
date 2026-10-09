function outer(statement) {
  function make() { return () => statement; }
  const get = make();
  const ignored = flag ? (statement = 'SELECT 1') : false;
  return get();
}
const result = outer('/* conditional annotation */ SELECT 1');
