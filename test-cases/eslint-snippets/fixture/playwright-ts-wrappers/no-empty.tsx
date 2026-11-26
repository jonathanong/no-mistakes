export const C = () => (
  <>
    <button data-pw="" />
    <button data-pw={"" as const} />
    <button data-pw={"" satisfies string} />
    <button data-pw={""!} />
    <button data-pw={("")} />
    <button data-pw={"valid" as const} />
  </>
);
