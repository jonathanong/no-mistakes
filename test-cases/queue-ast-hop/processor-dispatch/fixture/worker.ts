import { Worker } from 'bullmq';
import * as unrelated from './unrelated';
import literal from './literal';
import * as dispatchers from './literal';
import ambiguousDefault from './ambiguous-default';
import * as computedDynamic from './computed-dynamic';
import { makeWorker } from './factory';
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
// An inline namespace dispatcher must not adopt the module's static processor names.
new Worker('inline-namespace', (job) => dispatchers[job.name](job));
new Worker('ambiguous-default', ambiguousDefault);
new Worker('computed-dynamic', computedDynamic);
new Worker('namespace-member', dispatchers.default);
makeWorker('factory-dynamic', (job) => dispatchers[job.name](job));
makeWorker('factory-literal', dispatchers.default);
makeWorker('factory-default', literal);
