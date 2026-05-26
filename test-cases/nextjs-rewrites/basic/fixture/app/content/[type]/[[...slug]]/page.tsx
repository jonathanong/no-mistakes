import { notFound } from "next/navigation";

export default async function Page({
  params,
}: {
  params: Promise<{ type: string; slug?: string[] }>;
}) {
  const { type: contentType, slug = [] } = await params;
  if (contentType !== "posts" && contentType !== "reviews") {
    notFound();
  }
  return (
    <main>
      <h1>{contentType}</h1>
      <pre>{slug.join("/")}</pre>
    </main>
  );
}
