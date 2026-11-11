import { validate } from "./scan.mts";
const args = process.argv.slice(2);
validate({ root: args[0]!, paths: ["rules"] });
