export default { test: { projects: [
  { test: { name: 'compliant', isolate: true } },
  { test: { name: 'missing-required', pool: 'forks' } },
  { test: { name: 'wrong-literal', isolate: false } },
] } };
