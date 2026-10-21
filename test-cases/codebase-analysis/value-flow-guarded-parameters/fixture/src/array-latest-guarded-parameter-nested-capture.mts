import { primary, replica, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
function mutate(value) { value.client.get = replacement; }
function run(box) {
  function consume() {
    // Cross-context captures retain the existing conservative origin projection.
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
