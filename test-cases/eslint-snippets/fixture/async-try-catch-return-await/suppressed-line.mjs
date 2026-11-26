import errors from "@app/errors";

export async function send() {
  try {
    return sendMail(); // no-mistakes-disable-line async-try-catch-return-await -- retained probe
  } catch (error) {
    errors.report(error);
  }
}
