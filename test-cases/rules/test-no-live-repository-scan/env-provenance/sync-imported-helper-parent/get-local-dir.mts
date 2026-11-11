import { dirname } from "node:path";
export function getLocalDir() { return dirname(process.env.X ?? "./tmp/data"); }
