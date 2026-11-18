new Date().toLocaleString(void 0);
const absent = undefined;
new Date().toLocaleDateString(absent);
const alias = absent;
new Date().toLocaleTimeString(alias);
export function formatKnownUndefined() {
  const undefined = void 0;
  return new Date().toLocaleString(undefined);
}
