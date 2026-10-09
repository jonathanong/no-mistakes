const database = require('@app/db');
function createReceiver(statement) {
  const slots = arguments;
  function getSlots() {
    const saved = slots[0];
    const updated = (slots[0] = 'SELECT 2');
    database.query(saved); // finding:receiver-revisit
    return slots;
  }
  // Receiver evaluation invokes a helper that observes mutable lexical slots.
  return () => { const stored = (getSlots()[1] = 'unused'); };
}
const receiver = createReceiver('/* initial receiver */ SELECT 1');
const consumed = opaque(receiver);
