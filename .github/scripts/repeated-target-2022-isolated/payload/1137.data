import { suppliedParameter, defaultParameter, suppliedMulti, defaultMulti, parameterTdz } from '@vendor/valkey';
function replacement() {}
const suppliedSaved = suppliedParameter.get;
function acquireSupplied(fn = suppliedSaved) { return fn; }
(function parameterArgument(fn) { fn('key'); })(acquireSupplied(), suppliedParameter.get = replacement);
function acquireDefault(fn = defaultParameter.get) { return fn; }
(function parameterDefault(fn = acquireDefault()) { fn('key'); })(undefined, defaultParameter.get = replacement);
const multiSaved = suppliedMulti.get;
function acquireMulti(fn = multiSaved) { return fn; }
function forwardMulti() { return acquireMulti(); }
(function parameterMultiArgument(fn) { fn('key'); })(forwardMulti(), suppliedMulti.get = replacement);
function acquireMultiDefault(fn = defaultMulti.get) { return fn; }
function forwardMultiDefault() { return acquireMultiDefault(); }
(function parameterMultiDefault(fn = forwardMultiDefault()) { fn('key'); })(undefined, defaultMulti.get = replacement);
function acquireLate(fn = lateSaved) { return fn; }
(function parameterLate(fn = acquireLate()) { fn('key'); })();
const lateSaved = parameterTdz.get;
