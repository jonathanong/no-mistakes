import { primary, session, replica, scoped, local, standalone } from '@vendor/valkey';
function replacement() {}
function before(get = primary.get) { get('key'); }
before();
primary.get = replacement;
function after(get = session.get) { get('key'); }
session.get = replacement;
after();
function localWrite(get = replica.get) { replica.get = replacement; get('key'); }
localWrite();
function suppliedUndefined(get = scoped.get) { get('key'); }
suppliedUndefined(undefined);
scoped.get = replacement;
function fallback({ get = local.get } = {}) { get('key'); }
fallback({});
local.get = replacement;
function fallbackAfter({ get = standalone.get } = {}) { get('key'); }
standalone.get = replacement;
fallbackAfter({});
