import { useRouter } from 'next/navigation';
import { Card1 } from '@fixture/ui/components/Card1.tsx';
import { dataRecord1 } from '@fixture/data/records/data-1.mts';
import { clientCall1 } from '@fixture/http/client-1.mts';
export async function Feature1() {
  const router = useRouter();
  await clientCall1();
  router.push('/area-1/item/1');
  await fetch('/api/v1/resource-1/1');
  return <a href="/area-2/item/0"><Card1 record={dataRecord1} /></a>;
}
