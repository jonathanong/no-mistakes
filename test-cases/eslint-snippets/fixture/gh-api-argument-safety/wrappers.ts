declare function gh(args: unknown[]): void;
declare function ghApi(options: unknown, args: unknown[]): void;
gh(['api', 'repos/O/R/issues?bad=yes']);
ghApi({}, ['repos/O/R/issues', '-Fvalue=1']);
ghApi({}, ['graphql', '-Fvalue=1']);
function nested(gh: (x: unknown[]) => void) { gh(['api', 'repos/O/R/issues?bad=yes']); }
unconfigured(['api', 'repos/O/R/issues?bad=yes']);
const api = { gh() {} };
api.gh(['api', 'repos/O/R/issues?bad=yes']);
api['gh'](['api', 'repos/O/R/issues?bad=yes']);
api[key](['api', 'repos/O/R/issues?bad=yes']);
let replaced = gh;
replaced = other;
replaced(['api', 'repos/O/R/issues?bad=yes']);
gh(['api', 'repos/O/R/issues?bad=yes'] as const);
