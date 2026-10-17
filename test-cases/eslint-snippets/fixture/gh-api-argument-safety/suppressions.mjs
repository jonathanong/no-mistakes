import { execFile } from 'node:child_process';
// no-mistakes-disable-next-line gh-api-argument-safety -- reviewed endpoint
execFile('gh', ['api', 'repos/O/R/issues?bad=yes']);
execFile('gh', ['api', 'repos/O/R/issues?bad=yes']); // no-mistakes-disable-line no-mistakes/gh-api-argument-safety -- fixture
const text = 'no-mistakes-disable-file gh-api-argument-safety';
execFile('gh', ['api', 'repos/O/R/issues?bad=yes']);
