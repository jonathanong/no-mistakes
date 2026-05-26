import { useRouter } from 'next/navigation';
import { Card13 } from '@fixture/ui/components/Card13.tsx';
import { dataRecord13 } from '@fixture/data/records/data-13.mts';
import { clientCall13 } from '@fixture/http/client-13.mts';
export async function Feature13() {
  const router = useRouter();
  await clientCall13();
  router.push('/area-5/item/1');
  await fetch('/api/v1/resource-3/13');
  return <a href="/area-6/item/0"><Card13 record={dataRecord13} /></a>;
}
