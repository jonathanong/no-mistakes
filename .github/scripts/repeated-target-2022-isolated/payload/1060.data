import { primary } from '@vendor/valkey';
function replacement() {}
// An anonymous class field initializer runs for each new instance.
const Client = class {
  method = ((client = primary) => { const get = client.get; primary.get = replacement; get(); })();
};
new Client();
new Client();
