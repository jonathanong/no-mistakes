import { useRouter } from 'next/navigation';
import { Card5 } from '@fixture/ui/components/Card5.tsx';
import { dataRecord5 } from '@fixture/data/records/data-5.mts';
import { clientCall5 } from '@fixture/http/client-5.mts';
export async function Feature5() {
  const router = useRouter();
  await clientCall5();
  router.push('/area-5/item/1');
  await fetch('/api/v1/resource-5/5');
  return <a href="/area-6/item/0"><Card5 record={dataRecord5} /></a>;
}
