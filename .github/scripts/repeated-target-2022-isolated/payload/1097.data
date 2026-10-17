import { primary } from '@vendor/valkey';
function replacement() {}
declare const member: string;
// Ordinary module objects also keep computed-key callable bodies deferred.
const value = { [member]: () => ((client = primary) => { const get = client.get; primary.get = replacement; get(); })() };
