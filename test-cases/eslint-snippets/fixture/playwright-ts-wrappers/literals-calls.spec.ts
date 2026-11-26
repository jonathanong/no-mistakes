declare const page: any;
declare const dynamicId: string;

page.getByTestId("submit-btn" as const);
page.getByTestId(<const>"submit-btn");
(page.getByTestId as any)(dynamicId);
(<any>page.getByTestId)(dynamicId);
(page.getByTestId<any>)(dynamicId);
page["getByTestId" as const](dynamicId);
page.getByTestId.apply(page, [dynamicId] as const);
const { ["getByTestId" as const]: findById } = page;
findById(dynamicId);
page.getByTestId(dynamicId as string);
