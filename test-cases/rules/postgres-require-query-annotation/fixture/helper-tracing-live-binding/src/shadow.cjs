function outer(statement) {
  function make(statement) { return () => statement; }
  const get = make('/* local annotation */ SELECT 1');
  var statement = 'SELECT 1';
  return get();
}
const result = outer('/* outer annotation */ SELECT 1');
