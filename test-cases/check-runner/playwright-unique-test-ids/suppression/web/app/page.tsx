export default function Page() {
  const edit = <button data-testid="edit" />;
  const editDuplicate = <button data-testid="edit" />; // no-mistakes-disable-line playwright-unique-test-ids
  const save = <button data-testid="control" />;
  const saveDuplicate = <button data-testid="control" />;
  return <main>{edit}{editDuplicate}{save}{saveDuplicate}</main>;
}
