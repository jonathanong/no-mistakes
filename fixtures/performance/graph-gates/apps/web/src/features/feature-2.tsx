import { useRouter } from 'next/navigation';
import { Card2 } from '@fixture/ui/components/Card2';
import { dataRecord2 } from '@fixture/data/records/data-2';
import { clientCall2 } from '@fixture/http/client-2';
export async function Feature2() {
  const router = useRouter();
  await clientCall2();
  router.push('/area-2/item/0');
  await fetch('/api/v1/resource-2/2');
  return <a href="/area-0/item/1"><Card2 record={dataRecord2} /></a>;
}
