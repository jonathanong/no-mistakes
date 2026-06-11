const router = useRouter();
const maybePush = router?.push;
const maybePushCall = router.push?.();
