import { beginTransaction } from "@example/db";

export async function relatedDdl() {
  const tx = await beginTransaction();
  await tx.query("LOCK TABLE public.reservations IN SHARE MODE");
  await tx.query("DROP TABLE public.events");
}

export async function triggerWrite(id: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM public.accounts WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO public.orders (account_id) VALUES ($1)", [id]);
}
