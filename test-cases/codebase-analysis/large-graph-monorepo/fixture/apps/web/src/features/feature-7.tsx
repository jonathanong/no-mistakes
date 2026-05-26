import { useRouter } from 'next/navigation';
import { Card7 } from '@fixture/ui/components/Card7.tsx';
import { dataRecord7 } from '@fixture/data/records/data-7.mts';
import { clientCall7 } from '@fixture/http/client-7.mts';
export async function Feature7() {
  const router = useRouter();
  await clientCall7();
  router.push('/area-7/item/1');
  await fetch('/api/v1/resource-7/7');
  return <a href="/area-0/item/0"><Card7 record={dataRecord7} /></a>;
}
