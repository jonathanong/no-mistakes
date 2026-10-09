function create(parameter, removed) {
  const escaped = opaque(arguments);
  const deleted = delete arguments[1];
  // A definite parameter write remains fresh after the prior escape.
  var parameter = '/* fresh module callback */ SELECT 1';
  const sparse = (arguments[1000] = () => parameter);
  return () => parameter;
}
const callback = create('/* initial parameter */ SELECT 1', 'removed');
// A second retained invocation owns an ordinary dense-only argument object.
function simple(parameter) { return () => parameter; }
const simpleCallback = simple('/* dense-only callback */ SELECT 1');
// Keep this appended builder clean and reachable independently of opaque state.
const statement = sql`/* module builder */ SELECT 1`;
statement.append(' WHERE TRUE');
const appendedCallback = () => statement;
const dirty = sql`/* opaque module builder */ SELECT 1`;
const mutated = opaque(dirty);
