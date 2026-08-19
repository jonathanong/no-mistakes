// One file with many components: fused trait analysis is O(walks per file),
// not O(components). Splitting these into one component per file would hide
// the win this harness is meant to measure.
import { createContext, Suspense, useMemo, useState } from "react";

const Theme = createContext<null>(null);

function Leaf({ label }: { label: string }) {
  return <span>{label}</span>;
}

export function State0() {
  const [v, setV] = useState(0);
  return <button onClick={() => setV(v + 1)}>{v}</button>;
}
export function Props0({ label }: { label: string }) {
  return <span>{label}</span>;
}
export function Passes0(props: { label: string }) {
  return <Leaf {...props} />;
}
export function Memo0() {
  const v = useMemo(() => 1, []);
  return <span>{v}</span>;
}
export function Context0() {
  return (
    <Theme.Provider value={null}>
      <span />
    </Theme.Provider>
  );
}
export function Suspense0() {
  return (
    <Suspense fallback={null}>
      <div />
    </Suspense>
  );
}
export function Fetch0() {
  fetch("/api/0");
  return null;
}
export function Child0() {
  return <State0 />;
}

export function State1() {
  const [v, setV] = useState(1);
  return <button onClick={() => setV(v + 1)}>{v}</button>;
}
export function Props1({ label }: { label: string }) {
  return <span>{label}</span>;
}
export function Passes1(props: { label: string }) {
  return <Leaf {...props} />;
}
export function Memo1() {
  const v = useMemo(() => 2, []);
  return <span>{v}</span>;
}
export function Context1() {
  return (
    <Theme.Provider value={null}>
      <span />
    </Theme.Provider>
  );
}
export function Suspense1() {
  return (
    <Suspense fallback={null}>
      <div />
    </Suspense>
  );
}
export function Fetch1() {
  fetch("/api/1");
  return null;
}
export function Child1() {
  return <State1 />;
}

export function State2() {
  const [v, setV] = useState(2);
  return <button onClick={() => setV(v + 1)}>{v}</button>;
}
export function Props2({ label }: { label: string }) {
  return <span>{label}</span>;
}
export function Passes2(props: { label: string }) {
  return <Leaf {...props} />;
}
export function Memo2() {
  const v = useMemo(() => 3, []);
  return <span>{v}</span>;
}
export function Context2() {
  return (
    <Theme.Provider value={null}>
      <span />
    </Theme.Provider>
  );
}
export function Suspense2() {
  return (
    <Suspense fallback={null}>
      <div />
    </Suspense>
  );
}
export function Fetch2() {
  fetch("/api/2");
  return null;
}
export function Child2() {
  return <State2 />;
}

export function State3() {
  const [v, setV] = useState(3);
  return <button onClick={() => setV(v + 1)}>{v}</button>;
}
export function Props3({ label }: { label: string }) {
  return <span>{label}</span>;
}
export function Passes3(props: { label: string }) {
  return <Leaf {...props} />;
}
export function Memo3() {
  const v = useMemo(() => 4, []);
  return <span>{v}</span>;
}
export function Context3() {
  return (
    <Theme.Provider value={null}>
      <span />
    </Theme.Provider>
  );
}
export function Suspense3() {
  return (
    <Suspense fallback={null}>
      <div />
    </Suspense>
  );
}
export function Fetch3() {
  fetch("/api/3");
  return null;
}
export function Child3() {
  return <State3 />;
}
