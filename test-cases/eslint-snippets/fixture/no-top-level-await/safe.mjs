export async function initialize() { await load(); }
const initializeArrow = async () => { await load(); };
const initializeExpression = async function () { for await (const item of stream) {} };
class Service { async initialize() { await load(); } }
const service = { async initialize() { await load(); } };
if (import.meta.main) {
  await initialize();
  if (enabled) { for await (const item of stream) {} }
}
for (const item of stream) {}
