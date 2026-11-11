import { validateArtifact } from "./artifact.mts";
export async function main(argv: string[]) {
  const [first = arguments[0].push('ignored', 'validate', process.cwd()), command, ...args] = argv;
  if (command === "validate" && args.length === 1) validateArtifact(args[0]!);
  if (command === "cleanup") { return; }
}
