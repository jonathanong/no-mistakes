import { useRouter } from 'next/navigation';
import { Card8 } from '@fixture/ui/components/Card8.tsx';
import { dataRecord8 } from '@fixture/data/records/data-8.mts';
import { clientCall8 } from '@fixture/http/client-8.mts';
export async function Feature8() {
  const router = useRouter();
  await clientCall8();
  router.push('/area-0/item/0');
  await fetch('/api/v1/resource-8/8');
  return <a href="/area-1/item/1"><Card8 record={dataRecord8} /></a>;
}
