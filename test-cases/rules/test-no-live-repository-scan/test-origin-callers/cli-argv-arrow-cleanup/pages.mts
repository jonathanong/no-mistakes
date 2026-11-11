import { validateArtifact } from "./artifact.mts";
export const main = async (argv: string[]) => {
  const [command, ...args] = argv;
  if (command === "validate" && args.length === 1) validateArtifact(args[0]!);
  if (command === "cleanup") { return; }
};
