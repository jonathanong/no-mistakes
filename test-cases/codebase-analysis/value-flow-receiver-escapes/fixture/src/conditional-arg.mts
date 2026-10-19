import { primary, replica, untouched } from '@vendor/valkey';
function replacement() {}
declare const flag: boolean;
let box = { client: primary };
const other = { client: untouched };
function mutate(value) { value.client.get = replacement; }
box = { client: replica };
// Either branch can be the receiver. `untouched` may be unknown; `primary` must not.
mutate(flag ? other : box);
replica.get('changed');
untouched.get('other');
primary.get('old');
