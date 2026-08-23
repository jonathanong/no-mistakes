import { useRouter } from 'next/navigation';
import { Card5 } from '@fixture/ui/components/Card5';
import { dataRecord5 } from '@fixture/data/records/data-5';
import { clientCall2 } from '@fixture/http/client-2';
export async function Feature5() {
  const router = useRouter();
  await clientCall2();
  router.push('/area-2/item/1');
  await fetch('/api/v1/resource-2/5');
  return <a href="/area-0/item/0"><Card5 record={dataRecord5} /></a>;
}
