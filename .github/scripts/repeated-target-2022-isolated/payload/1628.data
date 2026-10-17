import { openTransaction } from "../transaction";

export async function anyModule() {
  const tx = await openTransaction();
  await tx(`SELECT id FROM relative_any`);
}
