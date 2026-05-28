import { useRouter } from "next/navigation";

export function UserLink({ id }: { id: string }) {
  const router = useRouter();
  const onClick = () => {
    router.push(`/v2/users/${id}`);
  };
  return <button onClick={onClick}>Go</button>;
}
