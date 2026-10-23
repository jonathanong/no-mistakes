// no-mistakes-disable-file gh-api-argument-safety -- protocol fixture
declare const ghApi: Function;
const alias = ghApi;
alias.call(null, 'unused', ['repos/O/R/labels?per_page=100']);
