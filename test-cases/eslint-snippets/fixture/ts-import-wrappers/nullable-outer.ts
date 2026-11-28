declare const dyn: string;
const m = require(dyn);
function computeId() {
  return "1";
}
export async function unrelated(): Promise<Widget | null> {
  return m.wrap(computeId());
}

declare const relayModule: string;
const relay = require(relayModule);
export async function wrapped(): Promise<Widget | null> {
  return wrapNullable(relay(getWidget()));
}
