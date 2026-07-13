// This ignored component must never enter the aggregate request snapshot.
export default async function Ignored() {
  const response = await fetch("/api/ignored");
  return <div>{response.status}</div>;
}
