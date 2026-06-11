export function entityHref(entity: { id: string }): string {
  return `/entities/${entity.id}`;
}

const router = useRouter();
router.push(entityHref(entity));
