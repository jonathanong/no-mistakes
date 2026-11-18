import { usePathname } from "next/navigation";

export function Metadata() {
  const pathname = usePathname();
  return (
    <script
      type="application/ld+json"
      data-path={pathname}
      dangerouslySetInnerHTML={{ __html: "{}" }}
    />
  );
}
