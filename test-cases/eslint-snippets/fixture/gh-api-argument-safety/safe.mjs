import { execFile as run } from 'node:child_process';
import * as child from 'node:child_process';
run('gh', ['api', '-X', 'GET', `repos/${repo}/labels`, '-F', 'per_page=100']);
run('gh', ['api', 'repos/O/R/issues', '--method=PATCH', '--raw-field=body=Ready?']);
run('gh', ['api', 'repos/O/R/issues', '-XGET', '-f', 'body=/Ready?']);
run('gh', ['api', 'graphql', '-F', 'query=query { viewer { login } }']);
run('gh', ['api', 'repos/O/R/issues', '--jq', '.foo?', '-q', '.bar?']);
run('gh', ['api', 'repos/O/R/issues', '--method', 'GET', '--field=value=1']);
run('gh', ['api', 'repos/O/R/issues', '-XHEAD', '-Fvalue=1']);
run('git', ['api', 'repos/O/R/issues?bad=yes']);
const unrelated = ['api', 'repos/O/R/issues?bad=yes'];
function shadow(run) { run('gh', ['api', 'repos/O/R/issues?bad=yes']); }
function shadowNamespace(child) { child.execFile('gh', ['api', 'repos/O/R/issues?bad=yes']); }
run('gh', ['api', ...args, 'repos/O/R/issues', '-Fvalue=1']);
run('gh', ['api', 'repos/O/R/issues', ...args, '-Fvalue=1']);
run('gh', dynamicArgs);
run('gh', ['api', 'repos/O/R/issues', '-Fvalue=1', '-X', method]);
run('gh', ['api', 'repos/O/R/issues', '--unknown', value, '-Fvalue=1']);
