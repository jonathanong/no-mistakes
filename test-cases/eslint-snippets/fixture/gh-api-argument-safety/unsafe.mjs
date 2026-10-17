import { execFile as run, spawnSync } from 'node:child_process';
import * as child from 'child_process';
run('gh', ['api', `repos/${repo}/labels?per_page=100`]);
spawnSync('gh', ['api', 'repos/O/R/labels', '-F', 'per_page=100']);
child.execFileSync('gh', ['api', '-fbody=Ready?', 'repos/O/R/issues']);
run('gh', ['api', 'repos/O/R/issues', '--raw-field=body=Ready?', '--method=']);
run('gh', ['api', 'repos/O/R/issues', '-Fvalue=1', '-X', '']);
run('gh', ['api', '-XGET', `repos/${repo}/labels?name=${name}&state=open`]);
run('gh', ['api', 'repos/O/R/issues', '--field', 'value=1']);
