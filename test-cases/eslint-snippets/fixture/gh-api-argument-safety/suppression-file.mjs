// no-mistakes-disable-file gh-api-argument-safety -- reviewed legacy request
import { execFile } from 'node:child_process';
execFile('gh', ['api', 'repos/O/R/issues?bad=yes', '-Fvalue=1']);
