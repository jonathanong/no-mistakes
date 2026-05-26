import { useRouter } from 'next/navigation';
import { Card16 } from '@fixture/ui/components/Card16.tsx';
import { dataRecord16 } from '@fixture/data/records/data-16.mts';
import { clientCall0 } from '@fixture/http/client-0.mts';
export async function Feature16() {
  const router = useRouter();
  await clientCall0();
  router.push('/area-0/item/0');
  await fetch('/api/v1/resource-6/16');
  return <a href="/area-1/item/1"><Card16 record={dataRecord16} /></a>;
}
