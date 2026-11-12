import { validateArtifact } from "./artifact.mts";
export async function main([first]: string[][]) {
  const [command, ...args] = first;
  if (command === "validate" && args.length === 1) validateArtifact(args[0]!);
  if (command === "cleanup") { return; }
}
