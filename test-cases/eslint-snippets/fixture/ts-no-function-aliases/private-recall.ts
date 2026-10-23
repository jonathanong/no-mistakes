type Input = { id: string };

declare const db: {
  createUser(input: Input): Input;
};
declare const otherDb: {
  createUser(input: Input): Input;
};
declare const nestedDb: {
  db: {
    createUser(input: Input): Input;
  };
};
declare function createUser(input: Input): Input;
declare function zeroArg(): void;
declare const rpc: {
  call(input: Input): Input;
  apply(input: Input): Input;
};

const dottedMember = (input: Input) => db.createUser(input);
const bracketMember = (input: Input) => db["createUser"](input);
const templateMember = (input: Input) => db[`createUser`](input);
const numericMember = (input: Input) => db[0](input);
const optionalMember = (input: Input) => db?.createUser(input);
const optionalCall = (input: Input) => db.createUser?.(input);

const templated = {};
templated[`createUser`] = (input: Input) => createUser(input);

const callForwarder = (input: Input) => createUser.call(undefined, input);
const voidCallForwarder = (input: Input) => createUser.call(void 0, input);
const applyForwarder = (input: Input) => createUser.apply(undefined, [input]);
const restApplyForwarder = (...args: Input[]) => createUser.apply(undefined, [...args]);
const memberCallForwarder = (input: Input) => db.createUser.call(db, input);
const memberApplyForwarder = (input: Input) => db.createUser.apply(db, [input]);
const nestedMemberCallForwarder = (input: Input) => nestedDb.db.createUser.call(nestedDb.db, input);
const rpcCallMethod = (input: Input) => rpc.call(input);
const rpcApplyMethod = (input: Input) => rpc.apply(input);
const zeroArgCallForwarder = () => zeroArg.call(undefined);
function recursiveCall(input: Input) {
  return recursiveCall.call(undefined, input);
}
function overloadedRecursive(input: string): string;
function overloadedRecursive(input: number): number;
function overloadedRecursive(input: string | number): string | number {
  return overloadedRecursive(input);
}
function zeroArgRecursiveCall() {
  return zeroArgRecursiveCall.call(undefined);
}
const recursiveApply = function recursiveApply(input: Input) {
  return recursiveApply.apply(undefined, [input]);
};

const api = {
  method(input: Input) {
    return input;
  },
};
api.method = (input: Input) => api.method(input);
const nested = {
  api: {
    method(input: Input) {
      return input;
    },
  },
};
nested.api.method = (input: Input) => nested.api.method(input);
const otherNested = {
  api: {
    method(input: Input) {
      return input;
    },
  },
};
nested.api.method = (input: Input) => otherNested.api.method(input);
const thisApi: { method?: (input: Input) => Input } = {};
thisApi.method = function (input: Input) {
  return thisApi.method!(input);
};
thisApi.method = (input: Input) => thisApi.method!(input);
function installThisMethod() {
  this.method = (input: Input) => this.method(input);
  this.method = function (input: Input) {
    return this.method(input);
  };
}

const differentReceiver = {};
differentReceiver.method = (input: Input) => api.method(input);

function createUser(input: Input) {
  return db.createUser(input);
}

const first = (input: Input) => second(input);
const second = (input: Input) => first(input);

function recur(recur: (input: Input) => Input) {
  return recur(recur);
}
function restRecur(...restRecur: Input[]) {
  return restRecur(...restRecur);
}
function defaultRecur(defaultRecur = createUser) {
  return defaultRecur(defaultRecur);
}
function objectRecur({ objectRecur }: { objectRecur: (input: Input) => Input }) {
  return objectRecur(objectRecur);
}
function arrayRecur([arrayRecur]: [(input: Input) => Input]) {
  return arrayRecur(arrayRecur);
}
function sparseRecur([, sparseRecur]: [unknown, (input: Input) => Input]) {
  return sparseRecur(sparseRecur);
}
function objectRestRecur({ ...objectRestRecur }: Record<string, (input: Input) => Input>) {
  return objectRestRecur(objectRestRecur);
}

const changesArguments = (input: Input) => db.createUser(input, { id: "audit" });
const changesReceiver = (input: Input) => db.createUser.call(otherDb, input);
const callWithDifferentReceiverType = (input: Input) => db.createUser.call(this, input);
const memberApplyChangesReceiver = (input: Input) => db.createUser.apply(otherDb, [input]);
const holeApply = (input: Input) => createUser.apply(undefined, [input, ,]);
const extraApplyArgument = (input: Input) => createUser.apply(undefined, [input], "extra");
const noCallReceiver = (input: Input) => createUser.call(input);
const missingCallReceiver = (input: Input) => createUser.call();
function shadowedUndefined(input: Input, undefined: unknown) {
  return createUser.call(undefined, input);
}
const dynamicAssignment = {};
dynamicAssignment[`create${"User"}`] = (input: Input) => createUser(input);
const dynamicMember = (input: Input, key: string) => db[key](input);
function dynamicThisTarget(input: Input) {
  return this(input);
}
const opaqueApply = (...args: Input[]) => createUser.apply(undefined, args);
