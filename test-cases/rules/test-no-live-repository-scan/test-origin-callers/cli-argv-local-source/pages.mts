import { validateArtifact } from "./artifact.mts";
const input = ["validate", process.cwd()];
export async function main(argv: string[]) {
  const [command, ...args] = input;
  if (command === "validate" && args.length === 1) validateArtifact(args[0]!);
  if (command === "cleanup") { return; }
}
