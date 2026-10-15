import { primary as client } from '@vendor/valkey';
function replacement() {}
export function terminalReceiver() {
  let box = { client };
  // Mutable-root projection reaches an imported terminal, which has no successors.
  box.client.get = replacement;
  const copied = box.client.get;
  // An unknown member may reach the same terminal through separate object paths.
  let diamond = { left: client, right: client };
  diamond[unknownKey].get = replacement;
  copied('saved');
  box.client.get('live');
}
