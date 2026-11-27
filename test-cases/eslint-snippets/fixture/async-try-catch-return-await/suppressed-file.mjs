// no-mistakes-disable-file async-try-catch-return-await -- retained probe
import errors from "@app/errors";

export async function send() {
  try {
    return sendMail();
  } catch (error) {
    errors.report(error);
  }
}
