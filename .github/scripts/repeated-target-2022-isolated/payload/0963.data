import { primary } from '@vendor/valkey';
function replacement() {}
// Baseline leaves these mutable lazy fields unresolved without receiver
// history; preserving that conservative boundary avoids extra cold-path frames.
let fieldSaved = primary.get;
function fieldAcquire() { return { get: fieldSaved }; }
(function mutableObjectArgument(object) { object.get('key'); })(fieldAcquire(), fieldSaved = replacement);
let fieldDefaultSaved = primary.get;
function fieldDefaultAcquire() { return { get: fieldDefaultSaved }; }
(function mutableObjectDefault(object = fieldDefaultAcquire()) { object.get('key'); })(undefined, fieldDefaultSaved = replacement);
