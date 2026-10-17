import { escapedReceiver, interruptedReceiver, rhs } from '@vendor/producer';
declare function retain(callback: () => void): void;
declare function interrupt(): void;
function escaped() {
  escapedReceiver.get('key');
  escapedReceiver.get = rhs.get;
}
retain(escaped);
escaped(); escaped();
function interrupted() {
  interruptedReceiver.get('key');
  interruptedReceiver.get = rhs.get;
}
interrupted(); interrupted(); interrupt(); interrupted();
