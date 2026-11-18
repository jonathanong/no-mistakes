export const ready = await initialize();
if (enabled) { await initialize(); }
for await (const item of stream) { consume(item); }
for (const item of await load()) { consume(item); }
class Service extends (await loadBase()) {
  [await key()]() {}
}
if (await enabled()) {}
