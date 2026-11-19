import { "beginTransaction" as openQuoted } from "@example/db";

export async function quotedFactory(name: string) {
  const tx = await openQuoted();
  await tx(`UPDATE widgets SET state = ${name}`);
}
