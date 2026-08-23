import { useRouter } from 'next/navigation';
import { Card4 } from '@fixture/ui/components/Card4';
import { dataRecord4 } from '@fixture/data/records/data-4';
import { clientCall1 } from '@fixture/http/client-1';
export async function Feature4() {
  const router = useRouter();
  await clientCall1();
  router.push('/area-1/item/0');
  await fetch('/api/v1/resource-1/4');
  return <a href="/area-2/item/1"><Card4 record={dataRecord4} /></a>;
}
