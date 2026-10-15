import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { wrapper: {} };
const retained = { client: primary };
box.wrapper = retained;
box.wrapper.client.get = replacement;

function consume(value, other) { value.get('original'); other.get('changed'); }
consume(primary, replica);
