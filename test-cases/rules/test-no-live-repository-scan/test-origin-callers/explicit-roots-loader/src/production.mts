import { describeFiles } from "./scan.mts";
export const command = () => describeFiles(process.cwd());
export const commands = [describeFiles];
