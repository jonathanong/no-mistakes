import { useRouter } from 'next/navigation';
import { Card10 } from '@fixture/ui/components/Card10.tsx';
import { dataRecord10 } from '@fixture/data/records/data-10.mts';
import { clientCall10 } from '@fixture/http/client-10.mts';
export async function Feature10() {
  const router = useRouter();
  await clientCall10();
  router.push('/area-2/item/0');
  await fetch('/api/v1/resource-0/10');
  return <a href="/area-3/item/1"><Card10 record={dataRecord10} /></a>;
}
