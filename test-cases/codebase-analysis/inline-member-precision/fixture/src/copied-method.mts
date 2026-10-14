import { primary } from '@vendor/valkey';
function replacement() {}
const get = primary.get;
// Invoke before the later write: earlier copied-argument timing is covered separately.
(({ method }) => method())({ method: get });
primary.get = replacement;
