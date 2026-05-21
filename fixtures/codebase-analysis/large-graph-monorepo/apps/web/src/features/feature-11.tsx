import { useRouter } from 'next/navigation';
import { Card11 } from '@fixture/ui/components/Card11.tsx';
import { dataRecord11 } from '@fixture/data/records/data-11.mts';
import { clientCall11 } from '@fixture/http/client-11.mts';
export async function Feature11() {
  const router = useRouter();
  await clientCall11();
  router.push('/area-3/item/1');
  await fetch('/api/v1/resource-1/11');
  return <a href="/area-4/item/0"><Card11 record={dataRecord11} /></a>;
}
