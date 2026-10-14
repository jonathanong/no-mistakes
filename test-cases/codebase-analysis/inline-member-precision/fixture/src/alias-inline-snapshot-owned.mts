// The copied method is acquired before this single inline body's write.
import { primary } from '@vendor/valkey';
function replacement() {}
((client = primary) => { const alias = client; const get = alias.get; primary.get = replacement; get(); })();
