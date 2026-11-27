declare const relayModule: string;
const relay = require(relayModule);
export async function knownMember(): Promise<Widget | null> {
  return relay(Math.abs(-1));
}
