declare const ghApi: Function;
declare const api: { gh: Function };
function shadow(ghApi: Function, api: { gh: Function }) {
  const alias = ghApi;
  alias('unused', ['repos/O/R/labels?per_page=100']);
  const { gh } = api;
  gh(['api', 'repos/O/R/labels?per_page=100']);
}
let replaced = ghApi;
replaced = another;
replaced('unused', ['repos/O/R/labels?per_page=100']);
const { [key]: computed } = api;
computed(['api', 'repos/O/R/labels?per_page=100']);
const { gh: fallback = another } = api;
fallback(['api', 'repos/O/R/labels?per_page=100']);
// Opaque argument lists do not prove which argument contains the argv.
ghApi.apply(null, args);
ghApi.call(...args);
const safe = ghApi;
safe('unused', ['repos/O/R/labels', '-X', 'GET', '-F', 'per_page=100']);
const unrelated = another;
unrelated('unused', ['repos/O/R/labels?per_page=100']);
const selected = api[key];
selected(['api', 'repos/O/R/labels?per_page=100']);
declare const declared: Function;
declared('unused', ['repos/O/R/labels?per_page=100']);
// Mutable bindings and default parameters do not prove wrapper identity.
let mutable = ghApi;
mutable('unused', ['repos/O/R/labels?per_page=100']);
function defaults({ gh } = api) { gh(['api', 'repos/O/R/labels?per_page=100']); }
const [arrayAlias] = [ghApi];
arrayAlias('unused', ['repos/O/R/labels?per_page=100']);
const { ...rest } = api;
rest.gh(['api', 'repos/O/R/labels?per_page=100']);
export { shadow, defaults };
// Chained intrinsic forwarding invokes the explicit receiver, not ghApi.
ghApi.call.call(other, null, 'unused', ['repos/O/R/labels?per_page=100']);
ghApi.apply.call(other, null, ['unused', ['repos/O/R/labels?per_page=100']]);
ghApi.call.apply(other, [null, 'unused', ['repos/O/R/labels?per_page=100']]);
ghApi.apply.apply(other, [null, ['unused', ['repos/O/R/labels?per_page=100']]]);
