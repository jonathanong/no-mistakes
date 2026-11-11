import { validateArtifact } from "./artifact.mts";
export async function main(argv: [number, ...string[]]) {
  const [command, ...args] = argv;
  if (command === 1 && args.length === 1) validateArtifact(args[0]!);
  if (command === 2) { return; }
}
