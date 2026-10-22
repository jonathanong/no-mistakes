const id = user.id;
page["getByTestId"](id);
page[`getByTestId`](id);
(page?.getByTestId)(id);
const { getByTestId } = page;
getByTestId(id);
page.getByTestId.call(page, id);
(0, page.getByTestId)(id);
page[name](id);
// no-mistakes-disable-next-line no-mistakes/playwright-literals
page.getByTestId(id);
