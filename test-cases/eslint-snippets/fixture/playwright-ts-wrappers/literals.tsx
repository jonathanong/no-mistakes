export const Good = () => <button data-pw={"submit-btn" as const} />;
export const GoodSatisfies = () => <button data-pw={"submit-btn" satisfies string} />;
export const GoodNonNull = () => <button data-pw={"submit-btn"!} />;
export const GoodDefault = ({ testId = "submit-btn" as const }) => (
  <button data-pw={testId as string} />
);
export const GoodProps = (props: { testId?: string }) => {
  const { testId = "submit-btn" as const } = props as { testId?: string };
  return <button data-pw={testId!} />;
};
export const Bad = ({ id }: { id: string }) => <button data-pw={id as string} />;
