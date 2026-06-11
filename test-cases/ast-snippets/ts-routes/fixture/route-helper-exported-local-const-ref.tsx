export const entityHref = (entity: { id: string }): string => `/entities/${entity.id}`;

const router = useRouter();
router.push(entityHref(entity));
