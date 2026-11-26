export const C = () => (
  <>
    <button data-pw="BadName" />
    <button data-pw={"BadName" as const} />
    <button data-pw={"BadName" satisfies string} />
    <button data-pw={"BadName"!} />
    <button data-pw={("BadName")} />
    <button data-pw={"good-name" as const} />
  </>
);
