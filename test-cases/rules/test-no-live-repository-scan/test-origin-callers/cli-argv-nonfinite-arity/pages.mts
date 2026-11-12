import { validateArtifact } from "./artifact.mts";
export async function main(argv: string[]) {
  const [command, ...args] = argv;
  if (args.length === 1e999) validateArtifact(args[0]!);
  if (command === "cleanup") { return; }
}
