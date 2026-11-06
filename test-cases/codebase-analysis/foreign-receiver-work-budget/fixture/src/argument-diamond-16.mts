import { provider0, provider1 } from './providers.mjs';
let box = { client: provider0 };
function write() {
  box.client.get = undefined;
  box.client.get = undefined;
  box.client.get = undefined;
  box.client.get = undefined;
}
function provide(client) { box.client = client; }
function forward0(client) { provide(client); provide(client); }
function forward1(client) { forward0(client); forward0(client); }
function forward2(client) { forward1(client); forward1(client); }
function forward3(client) { forward2(client); forward2(client); }
function forward4(client) { forward3(client); forward3(client); }
function forward5(client) { forward4(client); forward4(client); }
function forward6(client) { forward5(client); forward5(client); }
function forward7(client) { forward6(client); forward6(client); }
function forward8(client) { forward7(client); forward7(client); }
function forward9(client) { forward8(client); forward8(client); }
function forward10(client) { forward9(client); forward9(client); }
function forward11(client) { forward10(client); forward10(client); }
function forward12(client) { forward11(client); forward11(client); }
function forward13(client) { forward12(client); forward12(client); }
function forward14(client) { forward13(client); forward13(client); }
function forward15(client) { forward14(client); forward14(client); }
forward15(provider1);
write();
export { box };
