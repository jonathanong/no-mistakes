import { primary as receiver, replica } from '@vendor/database';
// Repeated calls revisit one physical imported-receiver occurrence.
function invoke() {
  receiver.read();
  receiver.read = replica.write;
}
invoke();
invoke();
