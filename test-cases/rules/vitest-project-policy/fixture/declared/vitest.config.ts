import { unitProject } from './helpers/projects.mts';
export default { test: { projects: names.map(unitProject) } };
export const nested = { settings: {
  fileParallelism: false,
  maxWorkers: -1,
  maxWorkers: 1_000,
  maxWorkers: +2,
  maxWorkers: (3 as number),
  maxWorkers: configuredWorkers,
  maxWorkers: settings.workers,
  maxWorkers: parseVitestMaxWorkers(),
  maxWorkers: -configuredWorkers,
  maxWorkers: !2,
  maxWorkers: -'2',
  maxWorkers: 1 + 2,
  // fileParallelism: false
  ['maxWorkers']: 9,
  maxWorkers() { return 2 },
  label: 'literal',
  enabled: true,
  retries: 2,
  mode: 'legacy',
  nullable: null,
} };
