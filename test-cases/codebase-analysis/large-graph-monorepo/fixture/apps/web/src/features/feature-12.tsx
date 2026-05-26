import { useRouter } from 'next/navigation';
import { Card12 } from '@fixture/ui/components/Card12.tsx';
import { dataRecord12 } from '@fixture/data/records/data-12.mts';
import { clientCall12 } from '@fixture/http/client-12.mts';
export async function Feature12() {
  const router = useRouter();
  await clientCall12();
  router.push('/area-4/item/0');
  await fetch('/api/v1/resource-2/12');
  return <a href="/area-5/item/1"><Card12 record={dataRecord12} /></a>;
}
