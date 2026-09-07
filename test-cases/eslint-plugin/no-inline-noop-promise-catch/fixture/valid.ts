export async function validCatches(
  work: Promise<void>,
  logError: (error: unknown) => void,
  logger: { warn: (error: unknown) => void },
) {
  work.catch(logError);
  work.catch(logger.warn);
  work.catch(function ignoreLockReleaseError() {});
  work.catch((error) => {
    console.log(error);
  });
  work.catch((error) => {
    report(error);
  });
  work.catch((error) => {
    throw error;
  });
  work.catch(() => null);
  work.catch(() => fallback);
  work.catch(() => false);
  work.then(onFulfilled);
  work.finally(() => {});
  work["catch"](() => {});
  work.catch(...handlers);
  work.catch((error) => error);
  work.catch(() => {
    return fallback;
  });
  work.catch(() => void 1);
  work.catch(() => void fallback);
  work.catch();
  work.catch(async (error) => {
    await report(error);
  });
  try {
    await work;
  } catch {
    /* not a Promise.catch */
  }
}

function report(error: unknown) {
  return error;
}

function onFulfilled() {}

const fallback = null;
const handlers: Array<(error: unknown) => void> = [];
