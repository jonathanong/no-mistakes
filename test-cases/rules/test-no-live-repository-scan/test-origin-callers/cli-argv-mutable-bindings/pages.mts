import { validateArtifact } from "./artifact.mts";
export async function main(argv: string[]) {
  let [command, ...args] = argv;
  command = "validate";
  args = [process.cwd()];
  if (command === "validate" && args.length === 1) validateArtifact(args[0]!);
  if (command === "cleanup") { return; }
}
