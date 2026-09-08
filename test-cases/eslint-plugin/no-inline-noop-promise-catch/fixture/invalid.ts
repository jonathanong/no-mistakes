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
  // Preserve this explicit EmptyStatement regression; formatters otherwise erase the node.
  // prettier-ignore
  work.catch(() => { ; });
  saveUser()
    .then((value) => value)
    .catch(() => {});
  work.catch(async () => {});
  lock.release().catch(() => {});
  (function () {
    return work;
  })().catch(() => {});
  work.catch(() => undefined as never);
  // Bare literals do not create a fallback or any other observable handling.
  work.catch(() => {
    0;
  });
  work.catch(() => {
    "ignored";
  });
  work.catch(() => {
    true;
  });
  work.catch(() => {
    null;
  });
  work.catch(() => {
    0n;
  });
  work.catch(() => {
    /ignored/;
  });
  work.catch(() => {
    `ignored`;
  });
}

function saveUser() {
  return Promise.resolve();
}

const lock = { release: () => Promise.resolve() };
