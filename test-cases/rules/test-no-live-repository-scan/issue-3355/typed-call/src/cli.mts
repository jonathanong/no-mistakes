import { scan } from "./scan.mts";

const index = process.argv.indexOf("--output-root");
scan(process.argv[index + 1]!);
