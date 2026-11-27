import errors from "@app/errors";

export async function send() {
  try {
    // no-mistakes-disable-next-line async-try-catch-return-await -- retained probe
    return sendMail();
  } catch (error) {
    errors.report(error);
  }
}
