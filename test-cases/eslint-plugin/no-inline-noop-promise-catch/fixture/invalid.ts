export async function invalidCatches(work: Promise<void>, ok: () => void) {
  work.catch(() => {});
  work.catch(() => {
    return;
  });
  work.catch(() => undefined);
  work.catch(() => void 0);
  work.catch(() => {
    return undefined;
  });
  work.catch(() => {
    return void 0;
  });
  work.then(ok, () => {});
  work.catch((_err) => {});
  work.catch(function () {});
  work.catch(() => {
    undefined;
  });
  work.catch(() => {
    void 0;
  });
  work.catch(() => {
    /* swallow */
  });
  work?.catch(() => {});
  work.catch(() => {
    ;
  });
  saveUser()
    .then((value) => value)
    .catch(() => {});
  work.catch(async () => {});
  lock.release().catch(() => {});
  (function () {
    return work;
  })().catch(() => {});
  work.catch(() => undefined as never);
}

function saveUser() {
  return Promise.resolve();
}

const lock = { release: () => Promise.resolve() };
