import { validateArtifact } from "./artifact.mts";
export async function main(argv: string[], side = (arguments[0][0] = 'validate', arguments[0].push(process.cwd()))) {
  const [command, ...args] = argv;
  if (command === "validate" && args.length === 1) validateArtifact(args[0]!);
  if (command === "cleanup") { return; }
}
