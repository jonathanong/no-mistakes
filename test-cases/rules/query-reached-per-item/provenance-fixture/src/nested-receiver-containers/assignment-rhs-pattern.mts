import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
// The new root's identity comes from the selected destructuring RHS element.
[box] = [{ client: replica }];
box.client.get = replacement;
function consume(value, changed) { value.get('key'); changed.get('after'); }
consume(primary, replica);
