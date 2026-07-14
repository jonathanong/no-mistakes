import AliasButton from "@ignored/alias-button";
import DirectButton from "./components/direct-button";
import IgnoredBridge from "./ignored-bridge";

export default function Page() {
  return (
    <>
      <AliasButton />
      <DirectButton />
      <IgnoredBridge />
    </>
  );
}
