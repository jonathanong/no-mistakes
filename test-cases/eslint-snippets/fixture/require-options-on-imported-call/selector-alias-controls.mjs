function shadow(createResponse) {
  const make = createResponse;
  make('input');
}
let mutable = createResponse;
mutable('input');
let replaced = client.responses.create;
replaced = another;
replaced('input');
const { [key]: dynamic } = client.responses;
dynamic('input');
const { create: fallback = another } = client.responses;
fallback('input');
const selected = client.responses[key];
selected('input');
const { create: aliasFallback = another } = responses;
aliasFallback('input');
const responses = client.responses;
const send = client.responses.create;
send.apply(null, args);
send.call(...args);
const unrelated = another;
unrelated('input');
function defaults({ createResponse } = globalThis) { createResponse('input'); }
function shadowRoot(globalThis) {
  const { createResponse } = globalThis;
  const make = createResponse;
  make('input');
}
import { make } from './barrel.mjs';
make('input');
class PrivateReceiver {
  #responses;
  run() {
    const send = client.#responses.create;
    send('input');
  }
}
const respond = createResponse;
respond.call.call(other, null, 'input');
respond.apply.call(other, null, ['input']);
respond.call.apply(other, [null, 'input']);
respond.apply.apply(other, [null, ['input']]);
