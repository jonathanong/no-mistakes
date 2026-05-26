import { useRouter } from 'next/navigation';
import { Card4 } from '@fixture/ui/components/Card4.tsx';
import { dataRecord4 } from '@fixture/data/records/data-4.mts';
import { clientCall4 } from '@fixture/http/client-4.mts';
export async function Feature4() {
  const router = useRouter();
  await clientCall4();
  router.push('/area-4/item/0');
  await fetch('/api/v1/resource-4/4');
  return <a href="/area-5/item/1"><Card4 record={dataRecord4} /></a>;
}
