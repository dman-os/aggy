import type { QueryClient } from "@tanstack/react-query";
// import { queryFamily } from "~/utils/atoms";

export class ApiClient {
  constructor(
    public aggyBaseUrl: string,
    public query: QueryClient,
  ) {
  }
}

export enum QUERY_KEYS {
  POSTS = 'POSTS',
  POST = 'POST',
}

type DePromisify<T> = T extends Promise<infer Inner> ? Inner : T;
type FnProps<T> = ({ [P in keyof T]: T[P] extends Function ? P : never })[keyof T];
type ApiResponse<FnName extends FnProps<ApiClient>> = DePromisify<ReturnType<ApiClient[FnName]>>;

// export const getProductsFam = queryFamily<string, ApiResponse<"getProducts">>(
//   (client, shopId) => ({
//     queryKey: [QUERY_KEYS.PRODUCTS, shopId],
//     queryFn: async ({ queryKey: [_, id] }) => client.getProducts(id as string),
//     // queryFn: async () => products,
//   })
// );
