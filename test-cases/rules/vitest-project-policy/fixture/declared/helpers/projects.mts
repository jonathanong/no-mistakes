export function unitProject(name: string) {
  return { test: {
    name,
    fileParallelism: false,
    'maxWorkers': 2,
    maxWorkers: parseVitestMaxWorkers(),
  } };
}
