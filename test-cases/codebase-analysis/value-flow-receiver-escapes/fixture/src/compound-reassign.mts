import { primary, replica, untouched } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(value) {
  // `+=` does not make the RHS a possible receiver identity.
  value += box;
  value.client.get = replacement;
}
box = { client: replica };
mutate({ client: untouched });
replica.get('changed');
primary.get('old');
untouched.get('argument');
