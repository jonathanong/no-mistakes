import { useRouter } from 'next/navigation';
import { Card3 } from '@fixture/ui/components/Card3';
import { dataRecord3 } from '@fixture/data/records/data-3';
import { clientCall0 } from '@fixture/http/client-0';
export async function Feature3() {
  const router = useRouter();
  await clientCall0();
  router.push('/area-0/item/1');
  await fetch('/api/v1/resource-0/3');
  return <a href="/area-1/item/0"><Card3 record={dataRecord3} /></a>;
}
