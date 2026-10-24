import { enqueueEmail } from "@app/jobs";
export default function () {
  return enqueueEmail(99);
}
