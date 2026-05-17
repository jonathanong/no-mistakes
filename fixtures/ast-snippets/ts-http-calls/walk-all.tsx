const top = client.get("/api/top");
const ignored = client.get("/other/top");

export const fromVar = client.post("/api/var");

export function exportedFunction() {
  return client.put("/api/exported-function");
}

export default () => {
  client.patch("/api/default-arrow");
};

export default function namedDefault() {
  client.delete("/api/default-function");
}

if (ready) {
  client.head("/api/if");
} else {
  client.options("/api/else");
}

try {
  fetch("/api/try");
} catch (error) {
  fetch("/api/catch");
}

for (let item = fetch("/api/for-init"); ready; ready = false) {
  fetch("/api/for-body");
}

for (const key in keys) {
  fetch("/api/for-in");
}

for (const value of values) {
  fetch("/api/for-of");
}

while (ready) {
  fetch("/api/while");
}

do {
  fetch("/api/do-while");
} while (ready);

const arrow = () => {
  fetch("/api/arrow");
};

const conditional = ready ? fetch("/api/conditional") : fetch("/api/alternate");
const logical = ready && fetch("/api/logical");
const sequence = (fetch("/api/sequence-one"), fetch("/api/sequence-two"));
const chained = client.wrap().get("/api/chained");
const casted = fetch("/api/casted") as unknown;
const nonNull = fetch("/api/non-null")!;
