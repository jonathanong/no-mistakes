import { beginTransaction } from "@sample/transactions";

export async function longChain() {
  const tx = await beginTransaction();
  await tx.a().b().c().d().e().f().g().h().i();
  await tx.commit();
}
