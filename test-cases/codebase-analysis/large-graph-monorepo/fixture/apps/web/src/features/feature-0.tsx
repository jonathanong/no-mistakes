import { useRouter } from 'next/navigation';
import { Card0 } from '@fixture/ui/components/Card0.tsx';
import { dataRecord0 } from '@fixture/data/records/data-0.mts';
import { clientCall0 } from '@fixture/http/client-0.mts';
export async function Feature0() {
  const router = useRouter();
  await clientCall0();
  router.push('/area-0/item/0');
  await fetch('/api/v1/resource-0/0');
  return <a href="/area-1/item/1"><Card0 record={dataRecord0} /></a>;
}
