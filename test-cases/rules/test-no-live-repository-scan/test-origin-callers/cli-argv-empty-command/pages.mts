import { validateArtifact } from "./artifact.mts";
export async function main(argv: string[]) {
  const [command, ...args] = argv;
  if (command === "validate") validateArtifact(args[0]!);
  if (command === "cleanup") { return; }
}
