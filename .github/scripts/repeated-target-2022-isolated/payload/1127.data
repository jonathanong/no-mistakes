import { primary, session, replica, scoped, local, standalone, nested } from '@vendor/valkey';
function replacement() {}
function factory() { return { get: primary.get }; }
const object = factory();
primary.get = replacement;
object.get('key');
function factoryAfter() { return { get: session.get }; }
session.get = replacement;
const objectAfter = factoryAfter();
objectAfter.get('key');
const copied = { get: replica.get };
replica.get = replacement;
function outsideFactory() { return copied; }
const outside = outsideFactory();
outside.get('key');
function localFactory() { const box = { get: scoped.get }; scoped.get = replacement; return box; }
const localObject = localFactory();
localObject.get('key');
function beforeFactory() { local.get = replacement; return { get: local.get }; }
const beforeObject = beforeFactory();
beforeObject.get('key');
function sharedFactory() { return { get: standalone.get }; }
function consume(options) { options.get('key'); }
consume(sharedFactory());
standalone.get = replacement;
consume(sharedFactory());
function manyFactory() { return { get: nested.get }; }
function manyConsume(options) { options.get('key'); }
manyConsume(manyFactory());
manyConsume(manyFactory());
nested.get = replacement;
function defaultConsume(options = object) { options.get('key'); }
defaultConsume();
