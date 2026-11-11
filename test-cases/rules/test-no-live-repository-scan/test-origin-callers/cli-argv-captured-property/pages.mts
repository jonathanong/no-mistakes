import { validateArtifact } from "./artifact.mts";
export async function main({ argv }: { argv: string[] }) {
  function inner(unused: string[]) {
    const [command, ...args] = argv;
    if (command === "validate" && args.length === 1) validateArtifact(args[0]!);
  }
  inner(["cleanup"]);
}
