import { cssBundleHref } from "@remix-run/css-bundle";
import type { LinksFunction, LoaderArgs } from "@remix-run/node";
import { json } from "@remix-run/node";
import {
  Links,
  LiveReload,
  Meta,
  Outlet,
  Scripts,
  ScrollRestoration,
  useLoaderData,
} from "@remix-run/react";
import { Provider, useAtom, createStore, } from "jotai"
import { useHydrateAtoms } from "jotai/utils";
import { queryClientAtom } from "jotai-tanstack-query"
import { Hydrate as QueryHydrate, QueryClientProvider } from "@tanstack/react-query"
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import { useDehydratedState } from 'use-dehydrated-state';
import React from "react";
import reset from "@unocss/reset/tailwind.css";

import global from "~/global.css"

import { aggyBaseUrlAtom } from "~/api/atoms";
import { AGGY_BASE_URL } from "~/api/constants.server";
import { ThemeHead, Theme, rawThemeAtom, themeAtom } from "~/utils/theme";
import { getThemeSession } from "~/utils/theme.server";


export const links: LinksFunction = () => [
  // NOTE: the ordering of these links matter
  { rel: "stylesheet", href: reset },
  { rel: "stylesheet", href: global },
  ...(cssBundleHref ? [{ rel: "stylesheet", href: cssBundleHref }] : []),
];

export const loader = async ({ request }: LoaderArgs) => {
  const themeSession = await getThemeSession(request);

  return json({
    ENV: {
      AGGY_BASE_URL
    },
    theme: dbg(themeSession.getTheme(), "loaded value")
  });
}

export default function App() {
  const loadedData = useLoaderData<typeof loader>();
  const [store] = React.useState(createStore())
  useHydrateAtoms([
    [aggyBaseUrlAtom, loadedData.ENV.AGGY_BASE_URL],
    [rawThemeAtom, loadedData.theme]
  ], {
    store
  });
  return (
    <Provider store={store}>
      <ProvidedApp />
    </Provider>
  );
}


function ProvidedApp() {
  let [queryClient] = useAtom(queryClientAtom);

  const [theme] = useAtom(themeAtom);

  const dehydratedState = useDehydratedState()

  return (
    <QueryClientProvider client={queryClient}>
      <QueryHydrate state={dehydratedState}>
        <html lang="en" className={theme == Theme.DARK ? 'dark' : ''}>
          <head>
            <meta charSet="utf-8" />
            <meta name="viewport" content="width=device-width,initial-scale=1" />
            <Meta />
            <Links />
            <ThemeHead hasCookieTheme={!!theme} />
          </head>
          <body>
            <Outlet />
            <ReactQueryDevtools initialIsOpen={false} />
            <ScrollRestoration />
            <Scripts />
            <LiveReload />
          </body>
        </html>
      </QueryHydrate>
    </QueryClientProvider >
  );
}

