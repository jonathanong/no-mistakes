import { validate } from "./scan.mts";
const args = process.argv.slice(2);
validate(args[0]!);
