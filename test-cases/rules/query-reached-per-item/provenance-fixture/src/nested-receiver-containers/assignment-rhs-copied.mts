import { primary as client } from '@vendor/valkey';
function replacement() {}
const box = { wrapper: {} };
const retained = { client };
const copied = client.get;
box.wrapper = retained;
box.wrapper.client.get = replacement;
function consume(value, fn) { value.get('live'); fn('snapshot'); }
consume(client, copied);
