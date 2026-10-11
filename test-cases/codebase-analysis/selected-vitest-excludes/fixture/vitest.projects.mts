// Both executions include the same test; only a excludes example.test.mts.
export const projects = [
  { test: { name: 'a', include: ['shared/*.test.mts'], exclude: ['shared/example.test.mts', 'shared/all-excluded.test.mts'], setupFiles: ['./setup/mock.mts'] } },
  { test: { name: 'b', include: ['shared/*.test.mts'], exclude: ['shared/all-excluded.test.mts'] } },
];
