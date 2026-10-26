import errors from "@app/errors";
import { report } from "@app/errors";

export async function byDefault() {
  try {
    return sendMail();
  } catch (error) {
    errors.report(error);
  }
}

export async function byCall() {
  try {
    return sendMail();
  } catch (error) {
    report.call(null, error);
  }
}
