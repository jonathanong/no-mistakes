import { primary } from '@vendor/valkey';
function replacement() {}
function run() {
  ((client = primary) => { const get = client.get; primary.get = replacement; get(); })();
}
// A later call to run can acquire the replacement.
run();
run();
