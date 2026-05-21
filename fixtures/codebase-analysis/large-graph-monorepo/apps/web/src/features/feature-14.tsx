import { useRouter } from 'next/navigation';
import { Card14 } from '@fixture/ui/components/Card14.tsx';
import { dataRecord14 } from '@fixture/data/records/data-14.mts';
import { clientCall14 } from '@fixture/http/client-14.mts';
export async function Feature14() {
  const router = useRouter();
  await clientCall14();
  router.push('/area-6/item/0');
  await fetch('/api/v1/resource-4/14');
  return <a href="/area-7/item/1"><Card14 record={dataRecord14} /></a>;
}
