declare const relayModule: string;
const relay = require(relayModule);
export async function load(): Promise<Widget | null> {
  return relay(getWidget());
}
declare function getWidget(): Promise<Widget | null>;
