import { Button as Action } from "./slot-components";
const Alias = Action;
<>
  <Button asChild data-pw="download">
    {/* Formatting and comments are not rendered children. */}
    <a href={url} download>Download</a>
  </Button>
  <Button asChild={true} data-testid={"save"}><button>Save</button></Button>
  <Button asChild="yes" data-pw={`name`}><input /></Button>
  <UI.Button asChild={1} data-pw="menu"><a href="/menu" /></UI.Button>
  <Action asChild data-pw="alias"><a href="/alias" /></Action>
  <Alias asChild data-pw="const-alias"><a href="/alias" /></Alias>
  <Button {...props} asChild data-pw="last"><a href="/last" /></Button>
  <Button asChild={true as const} data-pw={"typed" as const}><button /></Button>
  <Button asChild data-pw="own"><a {...props} data-pw="child" href="/own" /></Button>
  <button data-pw={dynamicId} />
  <Button asChild={`yes`} data-pw="template-slot"><button /></Button>
</>;
