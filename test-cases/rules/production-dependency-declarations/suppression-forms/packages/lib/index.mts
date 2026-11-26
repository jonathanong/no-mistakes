import chalk from "chalk";
// no-mistakes-disable-next-line production-dependency-declarations
import nextLine from "chalk";
import { fileSuppressed } from "./file-suppressed.mts";

export function helper() {
  return [chalk.red("control"), nextLine.red("next-line"), fileSuppressed()];
}
