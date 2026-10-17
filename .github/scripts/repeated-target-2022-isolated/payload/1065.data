import { primary } from '@vendor/valkey';
function replacement() {}
// Recursive calls may acquire an already replaced method.
export default function run(again = false) {
  ((client = primary) => { const get = client.get; primary.get = replacement; get(); })();
  if (again) run();
}
