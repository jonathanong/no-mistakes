declare const ghApi: Function;
declare const api: { gh: Function; call: Function; apply: Function };
const alias = ghApi;
alias('unused', ['repos/O/R/labels?per_page=100']);
ghApi.call(null, 'unused', ['repos/O/R/labels?per_page=100']);
ghApi.apply(null, ['unused', ['repos/O/R/labels', '-F', 'per_page=100']]);
const namespace = api;
namespace['gh'](['api', 'repos/O/R/labels?per_page=100']);
const { [`gh`]: extracted } = namespace;
extracted?.(['api', 'repos/O/R/labels?per_page=100']);
const member = api.gh;
(member as Function).call(null, ['api', 'repos/O/R/labels?per_page=100']);
(0, alias)('unused', ['repos/O/R/labels?per_page=100']);
// A configured method named call takes precedence over Function forwarding.
api.call(['api', 'repos/O/R/labels?per_page=100']);
function nested() {
  const local = ghApi;
  local('unused', ['repos/O/R/labels?per_page=100']);
}
const second = alias;
second['apply'](null, ['unused', ['repos/O/R/labels?per_page=100']]);
// no-mistakes-disable-next-line gh-api-argument-safety -- protocol fixture
alias('unused', ['repos/O/R/labels?per_page=100']);
alias('unused', ['repos/O/R/labels?per_page=100']); // no-mistakes-disable-line gh-api-argument-safety -- protocol fixture
export { nested };
api[`apply`](['api', 'repos/O/R/labels?per_page=100']);
