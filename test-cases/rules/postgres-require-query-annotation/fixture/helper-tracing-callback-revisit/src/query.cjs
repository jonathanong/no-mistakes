const database = require('@app/db');
function create(statement) {
  return () => {
    const saved = arguments[0];
    const updated = (arguments[0] = 'SELECT 1');
    database.query(saved); // finding:callback-revisit
  };
}
const reader = create('/* initial */ SELECT 1');
// The same consumer can call the callback again after its lexical slot changes.
const consumed = opaque(reader);
