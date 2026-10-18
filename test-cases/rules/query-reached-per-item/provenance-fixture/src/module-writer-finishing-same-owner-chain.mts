import { primary, replica } from '@vendor/valkey';

function replacement() {}
let other = { client: primary };
other = { client: replica };
let mid = { client: primary };
let box = { client: primary };

// `mid` is rebound by the writer itself, but `box` only reaches `mid` through
// `setup`, which is declared after `write`.
function write() {
  mid = other;
  box.client.get = replacement;
}

function setup() {
  box = mid;
}

write();
setup();
write();
primary.get('primary');
replica.get('replica was mutated by the second write');
