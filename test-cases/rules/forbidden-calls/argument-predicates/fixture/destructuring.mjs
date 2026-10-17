// The binding is not the whole initializer. Unsupported projections are unknown.
const [argv] = [['--root']];
const {cwd} = {cwd: '/live'};
execute(argv, cwd);
