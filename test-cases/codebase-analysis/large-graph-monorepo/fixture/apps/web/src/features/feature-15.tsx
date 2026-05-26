import { useRouter } from 'next/navigation';
import { Card15 } from '@fixture/ui/components/Card15.tsx';
import { dataRecord15 } from '@fixture/data/records/data-15.mts';
import { clientCall15 } from '@fixture/http/client-15.mts';
export async function Feature15() {
  const router = useRouter();
  await clientCall15();
  router.push('/area-7/item/1');
  await fetch('/api/v1/resource-5/15');
  return <a href="/area-0/item/0"><Card15 record={dataRecord15} /></a>;
}
