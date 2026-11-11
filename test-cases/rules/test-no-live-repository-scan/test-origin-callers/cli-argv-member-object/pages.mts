import { validateArtifact } from "./artifact.mts";
export async function main(argv: string[]) {
  const [command, ...args] = argv;
  if (argv.slice(0).length === 2) validateArtifact(args[0]!);
  if (command === "cleanup") { return; }
}
