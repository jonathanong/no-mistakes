export const Page = () => (
  <>
    <button data-testid="save" />
    <button data-testid={"save" as const} />
    <button data-testid={"save" satisfies string} />
    <button data-testid={"save"!} />
    <button data-testid={("save")} />
    <button data-testid={"other" as const} />
  </>
);
