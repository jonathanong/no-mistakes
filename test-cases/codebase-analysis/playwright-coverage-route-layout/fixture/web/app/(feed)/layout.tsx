import { PageWithAside } from "../../components/page-with-aside";
export default function FeedLayout({ children }: { children: React.ReactNode }) {
  return <PageWithAside>{children}</PageWithAside>
}
