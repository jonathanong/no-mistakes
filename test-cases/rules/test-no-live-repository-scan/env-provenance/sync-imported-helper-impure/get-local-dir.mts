export function getLocalDir() { process.env.X = process.cwd(); return process.env.X ?? "./tmp/data"; }
