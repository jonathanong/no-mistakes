// Exported const cells do not freeze mutable argument carriers.
export const argv = ['--root'];
export const options = {cwd: '/workspace'};
export const command = 'node';
export function invoke() { execute(command, argv, options); }
