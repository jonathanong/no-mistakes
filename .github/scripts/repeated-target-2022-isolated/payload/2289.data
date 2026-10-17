import { primary, supplied, defaultLater, suppliedLater, forwarded, objectSource, snapshot, tdz, mutableBefore, mutableAfter, objectLater } from '@vendor/valkey';
function replacement() {}

function acquire() { return saved; }
const saved = primary.get;
(function nestedDefault(fn = acquire()) { fn('key'); })();

function suppliedAcquire() { return suppliedSaved; }
const suppliedSaved = supplied.get;
(function nestedArgument(fn) { fn('key'); })(suppliedAcquire(), supplied.get = replacement);

function defaultMethodAcquire() { return defaultLater.get; }
(function nestedDefaultLater(fn = defaultMethodAcquire()) { fn('key'); })(undefined, defaultLater.get = replacement);
function argumentMethodAcquire() { return suppliedLater.get; }
(function nestedArgumentLater(fn) { fn('key'); })(argumentMethodAcquire(), suppliedLater.get = replacement);

function forward(fn) { return fn; }
function forwardAcquire() { return forward(forwardSaved); }
const forwardSaved = forwarded.get;
(function forwardedDefault(fn = forwardAcquire()) { fn('key'); })();

function objectAcquire() { return { fn: objectSaved }; }
const objectSaved = objectSource.get;
(function objectDefault({ fn } = objectAcquire()) { fn('key'); })();

function snapshotAcquire() { return snapshotSaved; }
const snapshotSaved = snapshot.get;
(function snapshotDefault(fn = snapshotAcquire()) { fn('key'); })(undefined, snapshot.get = replacement);

function unavailableAcquire() { return unavailable; }
(function declarationAfterCall(fn = unavailableAcquire()) { fn('key'); })();
const unavailable = tdz.get;

function localUnavailableAcquire() { return localUnavailable; const localUnavailable = primary.get; }
(function localDeclarationAfterReturn(fn = localUnavailableAcquire()) { fn('key'); })();

function mutableAcquire() { return mutableSaved; }
function copyMutableBefore() { return mutableBefore.get; }
let mutableSaved = copyMutableBefore();
(function mutableArgument(fn) { fn('key'); })(mutableAcquire(), mutableSaved = replacement);
function mutableDefaultAcquire() { return mutableDefaultSaved; }
function copyMutableAfter() { return mutableAfter.get; }
let mutableDefaultSaved = copyMutableAfter();
(function mutableDefault(fn = mutableDefaultAcquire()) { fn('key'); })(undefined, mutableDefaultSaved = replacement);

function laterObjectAcquire() { return { fn: objectLater.get }; }
(function objectDefaultLater({ fn } = laterObjectAcquire()) { fn('key'); })(undefined, objectLater.get = replacement);

function enclosingFactory() {
  function enclosingAcquire() { return enclosingSaved; }
  const enclosingSaved = primary.get;
  return function enclosingDefault(fn = enclosingAcquire()) { fn('key'); };
}
enclosingFactory()();
