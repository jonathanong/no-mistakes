import { describeFiles } from "./scan.mts";
export default function setup() {
  const scan = describeFiles;
  return scan(process.cwd());
}
