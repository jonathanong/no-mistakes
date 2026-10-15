import { lateAgreementReceiver, copiedAgreementReceiver } from '@vendor/foreign-parameter-late';
const copied = copiedAgreementReceiver.get;
export function acquire(method = lateAgreementReceiver.get) { return method; }
export function acquireCopied(method = copied) { return method; }
