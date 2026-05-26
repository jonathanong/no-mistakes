import { useRouter } from 'next/navigation';
import { Card3 } from '@fixture/ui/components/Card3.tsx';
import { dataRecord3 } from '@fixture/data/records/data-3.mts';
import { clientCall3 } from '@fixture/http/client-3.mts';
export async function Feature3() {
  const router = useRouter();
  await clientCall3();
  router.push('/area-3/item/1');
  await fetch('/api/v1/resource-3/3');
  return <a href="/area-4/item/0"><Card3 record={dataRecord3} /></a>;
}
