import { objectEscapeSnapshot, returnedEscapeSnapshot } from '@vendor/valkey';
function replacement() {}
function writeObjectEscape() { objectEscapeSnapshot.get = replacement; }
// The opaque supplier can invoke the writer before the method is copied.
external({ write: writeObjectEscape });
const objectSaved = objectEscapeSnapshot.get;
(function savedObjectEscape(fn) { fn('key'); })(objectSaved);
writeObjectEscape();

function writeReturnedEscape() { returnedEscapeSnapshot.get = replacement; }
function returnEscapedWriter() { return writeReturnedEscape; }
external(returnEscapedWriter());
const returnedSaved = returnedEscapeSnapshot.get;
(function savedReturnedEscape(fn) { fn('key'); })(returnedSaved);
writeReturnedEscape();
