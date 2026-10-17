// Exporting an alias exposes the same mutable carrier.
const argv = ['--root'];
const alias = argv;
export {alias};
execute(argv);
const command = 'node';
export {command};
execute(command);
