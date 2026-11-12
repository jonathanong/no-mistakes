import { validateArtifact } from "./artifact.mts";
export async function main(argv: [string, string]) {
  const [command, ...{ 0: args }] = argv;
  if (command === "validate" && args.length === 1) validateArtifact(args);
  if (command === "cleanup") { return; }
}
