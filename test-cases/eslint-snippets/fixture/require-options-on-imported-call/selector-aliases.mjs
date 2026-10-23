createResponse('input');
const make = createResponse;
make('input');
client.responses.create('input');
const send = client.responses.create;
send('input');
const responses = client.responses;
responses.create('input');
const { create } = responses;
create('input');
make.call(null, 'input');
send['apply'](null, ['input']);
const second = make;
(0, second)?.('input');
const { [`createResponse`]: respond } = globalThis;
const again = respond;
again('input');
// A configured literal receiver method call retains its original arguments.
const invoke = client.call;
invoke('input');
// no-mistakes-disable-next-line require-options-on-imported-call -- protocol fixture
make('input');
make('input'); // no-mistakes-disable-line require-options-on-imported-call -- protocol fixture
make('input', { safe: true });
client.call.call(null, 'input');
client.call.apply(null, ['input']);
make.call.call(make, null, 'input');
make.apply.call(make, null, ['input']);
make.call.apply(make, [null, 'input']);
make.apply.apply(make, [null, ['input']]);
(0, make.call).call(make, null, 'input');
