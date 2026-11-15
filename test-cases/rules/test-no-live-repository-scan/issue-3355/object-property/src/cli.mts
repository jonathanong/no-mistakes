import { main } from "./scan.mts";

const index = process.argv.indexOf("--output-root");
main({ outputRoot: process.argv[index + 1]! });
