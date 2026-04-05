declare function graphsResponse({}: {}, use: any): Promise<void>;
declare function graphResponses({}: {}, use: any): Promise<void>;
declare const apiTest: import("vitest").TestAPI<{
    graphsResponse: typeof graphsResponse;
    graphResponses: typeof graphResponses;
}>;
export { apiTest };
