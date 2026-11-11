import { validateArtifact } from "./artifact.mts";
export async function main(argv: string[]) {
  const command = "validate";
  if (command === "validate") validateArtifact(process.cwd());
  if (command === "cleanup") { return; }
}
