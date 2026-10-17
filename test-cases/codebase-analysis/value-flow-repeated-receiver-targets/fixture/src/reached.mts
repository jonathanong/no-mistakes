import { primary, replica } from '@vendor/valkey';
function run() {
  primary.get('key');
  primary.get = replica.get;
}
for (const item of ['a', 'b']) {
  run();
  run();
}
