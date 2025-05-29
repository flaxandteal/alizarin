declare function graphsResponse({ task }: {
    task: any;
}, use: any): Promise<void>;
declare function graphResponses({ task }: {
    task: any;
}, use: any): Promise<void>;
declare const apiTest: import('vitest').TestAPI<{
    graphsResponse: typeof graphsResponse;
    graphResponses: typeof graphResponses;
}>;
export { apiTest };
