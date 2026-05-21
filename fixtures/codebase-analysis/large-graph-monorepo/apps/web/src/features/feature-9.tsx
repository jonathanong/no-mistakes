import { useRouter } from 'next/navigation';
import { Card9 } from '@fixture/ui/components/Card9.tsx';
import { dataRecord9 } from '@fixture/data/records/data-9.mts';
import { clientCall9 } from '@fixture/http/client-9.mts';
export async function Feature9() {
  const router = useRouter();
  await clientCall9();
  router.push('/area-1/item/1');
  await fetch('/api/v1/resource-9/9');
  return <a href="/area-2/item/0"><Card9 record={dataRecord9} /></a>;
}
