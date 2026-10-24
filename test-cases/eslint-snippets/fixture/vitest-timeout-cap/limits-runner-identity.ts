import runner from 'vitest';
import { remoteTest } from './limits-runner-source';

// Vitest's default import is not a runner; another file's export is unresolved.
runner('default import', () => {}, 60_000);
runner.it('default member', () => {}, 60_000);
remoteTest('cross-file runner', () => {}, 60_000);
