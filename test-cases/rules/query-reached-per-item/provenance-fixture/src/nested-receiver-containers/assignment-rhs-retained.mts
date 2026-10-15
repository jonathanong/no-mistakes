import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
const box = { wrapper: {} };
const retained = { client };
// Replacement creates a live receiver alias; the declaration has no client field.
box.wrapper = retained;
box.wrapper.client.get = replacement;
function consume(value, untouched) { value.get('key'); untouched.get('sibling'); }
consume(client, replica);
