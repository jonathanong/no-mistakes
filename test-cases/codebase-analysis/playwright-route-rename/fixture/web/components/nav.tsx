import { useRouter } from "next/navigation";

export function Nav() {
  const router = useRouter();
  const onClick = () => {
    router.push("/v2/dashboard");
  };
  return <button onClick={onClick}>Go</button>;
}
