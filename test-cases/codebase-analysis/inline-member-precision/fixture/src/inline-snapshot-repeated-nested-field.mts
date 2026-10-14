import { primary } from '@vendor/valkey';
function replacement() {}
// This anonymous class has no direct class binding, yet its field runs per instance.
const holder = {
  Client: class {
    method = ((client = primary) => { const get = client.get; primary.get = replacement; get(); })();
  },
};
new holder.Client();
new holder.Client();
