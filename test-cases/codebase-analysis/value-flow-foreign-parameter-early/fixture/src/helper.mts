import { earlyAgreementReceiver, copiedAgreementReceiver } from '@vendor/foreign-parameter-early';
const copied = copiedAgreementReceiver.get;
export function acquire(method = earlyAgreementReceiver.get) { return method; }
export function acquireCopied(method = copied) { return method; }
