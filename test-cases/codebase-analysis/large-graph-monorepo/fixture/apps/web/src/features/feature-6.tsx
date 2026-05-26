import { useRouter } from 'next/navigation';
import { Card6 } from '@fixture/ui/components/Card6.tsx';
import { dataRecord6 } from '@fixture/data/records/data-6.mts';
import { clientCall6 } from '@fixture/http/client-6.mts';
export async function Feature6() {
  const router = useRouter();
  await clientCall6();
  router.push('/area-6/item/0');
  await fetch('/api/v1/resource-6/6');
  return <a href="/area-7/item/1"><Card6 record={dataRecord6} /></a>;
}
