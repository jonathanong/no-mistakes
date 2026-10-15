import { primary } from '@vendor/valkey';
function replacement() {}
// An anonymous auto-accessor initializer runs for each new instance.
const Client = class {
  accessor method = ((client = primary) => { const get = client.get; primary.get = replacement; get(); })();
};
new Client();
new Client();
