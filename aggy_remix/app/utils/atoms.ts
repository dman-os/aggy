
import { useAtom } from "jotai";
import { atomFamily, } from "jotai/utils";
import { atomsWithQuery, } from "jotai-tanstack-query";
import type { QueryKey, QueryOptions } from "@tanstack/react-query";
import { apiClientAtom } from "~/api/atoms";
import type { ApiClient } from "~/api";

export function queryFamily<
  P = unknown,
  TQueryFnData = unknown,
  TError = unknown,
  TData = unknown,
  TQueryKey extends QueryKey = QueryKey,
>(
  queryOptFn: (client: ApiClient, params: P) => QueryOptions<TQueryFnData, TError, TData, TQueryKey>,
  // queryOptFn: (args: { client: ApiClient, params: P }) => QueryOptions<TQueryFnData, TError, TData, TQueryKey>,
) {
  const family = atomFamily(
    (params: P) => {
      const [queryAtom] = atomsWithQuery(
        // (get) => queryOptFn({ params, client: get(apiClientAtom) })
        (get) => queryOptFn(get(apiClientAtom), params)
      );
      return queryAtom!;
    },
  );
  family.setShouldRemove((ts, _p) => Date.now() - ts > 60 * 60 * 1000);
  return {
    atomFamily: family,
    prefetch: async (client: ApiClient, params: P) => {
      await client.query.prefetchQuery(queryOptFn(client, params));
    },
    // prefetch: async (args: { client: ApiClient, params: P }) => {
    //   await args.client.query.prefetchQuery(queryOptFn(args));
    // },
    use: (params: P) => {
      const atom = family(params);
      // atom.onMount = (_) => () => fam.atomFamily.remove(params);
      // eslint-disable-next-line react-hooks/rules-of-hooks
      return useAtom(atom);
    }
  } as const;
}

export function useQueryFam<
  P, TQueryFnData, TError, TData, TQueryKey extends QueryKey,
>(
  fam: ReturnType<typeof queryFamily<P, TQueryFnData, TError, TData, TQueryKey>>,
  params: P,
) {
  const atom = fam.atomFamily(params);
  // atom.onMount = (_) => () => fam.atomFamily.remove(params);
  return useAtom(atom);
}
/*
export function mutationFamily<
  P = unknown,
  TQueryFnData = unknown,
  TError = unknown,
  TData = unknown,
  TQueryKey extends QueryKey = QueryKey,
>(
  mutationOptFn: (client: ApiClient, params: P) => QueryOptions<TQueryFnData, TError, TData, TQueryKey>,
  // queryOptFn: (args: { client: ApiClient, params: P }) => QueryOptions<TQueryFnData, TError, TData, TQueryKey>,
) {
  const family = atomFamily(
    (params: P) => {
      const [queryAtom] = atomsWithMutation(
        // (get) => queryOptFn({ params, client: get(apiClientAtom) })
        (get) => mutationOptFn(get(apiClientAtom), params)
      );
      return queryAtom!;
    },
  );
  family.setShouldRemove((ts, _p) => Date.now() - ts > 60 * 60 * 1000);
  return {
    atomFamily: family,
    prefetch: async (client: ApiClient, params: P) => {
      await client.query.prefetchQuery(mutationOptFn(client, params));
    },
    // prefetch: async (args: { client: ApiClient, params: P }) => {
    //   await args.client.query.prefetchQuery(queryOptFn(args));
    // },
    use: (params: P) => {
      const atom = family(params);
      // atom.onMount = (_) => () => fam.atomFamily.remove(params);
      // eslint-disable-next-line react-hooks/rules-of-hooks
      return useAtom(atom);
    }
  } as const;
}

export function useMutationFam<
  P, TQueryFnData, TError, TData, TQueryKey extends QueryKey,
>(
  fam: ReturnType<typeof queryFamily<P, TQueryFnData, TError, TData, TQueryKey>>,
  params: P,
) {
  const atom = fam.atomFamily(params);
  // atom.onMount = (_) => () => fam.atomFamily.remove(params);
  return useAtom(atom);
} */
