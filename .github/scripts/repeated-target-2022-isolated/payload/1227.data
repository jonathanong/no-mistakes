import { primary as receiver, read, ignored } from "@vendor/plain-alternatives";
for (const item of ["a", "b"]) {
  receiver.transaction(item);
  receiver.transaction = read;
}
ignored();
