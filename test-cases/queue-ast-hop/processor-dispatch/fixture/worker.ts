import { Worker } from 'bullmq';
import * as unrelated from './unrelated';
import literal from './literal';
import * as mixed from './mixed';
import * as computed from './computed';
import * as comparison from './comparison';
import * as cases from './cases';
import * as ambiguous from './ambiguous';
new Worker('unrelated', unrelated);
new Worker('literal', literal);
new Worker('mixed', mixed);
new Worker('computed', computed);
new Worker('comparison', comparison);
new Worker('cases', cases);
new Worker('inline', (job) => ({ name: 'result', data: job.data }), { name: 'options' });
new Worker('inline-literal', (task) => {
  // The processor parameter can use another name.
  if (task.name === 'alpha') return;
});
new Worker('inline-mixed', ({ name: jobName }) => {
  if (jobName === 'known') return;
  return handlers[jobName]();
});
declare const handlers: Record<string, Function>;
new Worker('ambiguous', ambiguous);
