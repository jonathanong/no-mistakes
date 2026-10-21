import { primary, replica, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
function mutate(value) { value.client.get = replacement; }
function run(box) {
  function consume() {
    // This once-called helper always writes replica; possible origins record
    // the existing conservative cross-context projection, not runtime alternatives.
    let args = [box];
    if (flag) { args = [box]; }
    mutate(...args);
  }
  consume();
}
run({ client: replica });
replica.get('nested capture remains possible');
replica.info('sibling control');
primary.get('unrelated origin');
unrelated.get('control');
