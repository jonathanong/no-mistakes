export function run(
  work: Promise<void>,
  releaseLock: () => Promise<void>,
  saveUser: () => Promise<void>,
) {
  work.catch(() => {});
  releaseLock().catch(() => {});
  saveUser().catch(() => {});
  releaseLock()
    .then((value) => value)
    .catch(() => {});
}
