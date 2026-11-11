import * as path from "node:path";
export function getLocalDir() { return path.resolve(process.env.X, "/unowned"); }
