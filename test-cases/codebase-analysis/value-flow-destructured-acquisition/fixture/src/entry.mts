import { primary, session } from '@vendor/valkey';
function replacement() {}
const opts = { client: primary };
const copied = { client: { get: primary.get } };
function beforeWrite({ client: { get } }) { get('key'); }
beforeWrite(opts);
primary.get = replacement;
function afterWrite({ client: { get } }) { get('key'); }
afterWrite(opts);
function copiedMethod({ client: { get } }) { get('key'); }
copiedMethod(copied);
function untouched({ client: { info } }) { info('key'); }
untouched(opts);
function nested({ box: { client: { get } } }) { get('key'); }
nested({ box: opts });
function forwarded({ client: { get } }) { get('key'); }
function forward(options) { forwarded(options); }
forward(opts);
function defaulted({ client: { get } } = opts) { get('key'); }
defaulted();
function arrayInput([{ client: { get } }]) { get('key'); }
arrayInput([opts]);
// The method is acquired in the callee default before the body replaces it.
function ownedDefault({ client: { get } } = { client: session }) {
  session.get = replacement;
  get('key');
}
ownedDefault();
