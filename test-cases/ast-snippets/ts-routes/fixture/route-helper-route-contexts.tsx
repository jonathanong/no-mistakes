import { redirect } from 'next/navigation';
import { entityHref } from './entity-href';
import { type Entity } from './entity-href';

const loose = entityHref(entity);
api.fetch(entityHref(entity));
const link = <Link href={entityHref(entity)} />;
const router = useRouter();
router.push(entityHref(entity));
redirect(entityHref(entity));
fetch(entityHref(entity));
router?.push(entityHref(entity));
router.push?.(entityHref(entity));
redirect?.(entityHref(entity));
globalThis?.fetch(entityHref(entity));
router?.[method](entityHref(entity));
router.push(entityHref?.(entity));
const optionalMember = links?.entityHref;
for (const item of items) {
  router.prefetch(entityHref(item));
}
while (next) router.replace(entityHref(next));
switch (tab) {
  case 'details':
    router.push(entityHref(entity));
    break;
}
try {
  router.prefetch(entityHref(entity));
} catch {
  router.replace(entityHref(entity));
} finally {
  redirect(entityHref(entity));
}
async function navigate() {
  await router.push(entityHref(entity));
  router.push(await entityHref(entity));
  router.replace(withLocale(entityHref(entity)));
}
