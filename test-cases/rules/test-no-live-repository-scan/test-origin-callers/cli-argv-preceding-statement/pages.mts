import { validateArtifact } from "./artifact.mts";
export async function main(argv: string[]) {
  void 0;
  const [command, ...args] = argv;
  if (command === "validate" && args.length === 1) validateArtifact(args[0]!);
  if (command === "cleanup") { return; }
}
