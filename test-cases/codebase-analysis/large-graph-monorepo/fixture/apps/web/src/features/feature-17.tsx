import { useRouter } from 'next/navigation';
import { Card17 } from '@fixture/ui/components/Card17.tsx';
import { dataRecord17 } from '@fixture/data/records/data-17.mts';
import { clientCall1 } from '@fixture/http/client-1.mts';
export async function Feature17() {
  const router = useRouter();
  await clientCall1();
  router.push('/area-1/item/1');
  await fetch('/api/v1/resource-7/17');
  return <a href="/area-2/item/0"><Card17 record={dataRecord17} /></a>;
}
