import { AGGY_BASE_URL } from "./constants.server"
import { QueryClient } from "@tanstack/react-query";
import { json, redirect } from "@remix-run/node";
import { dehydrate } from '@tanstack/react-query'

import { ApiClient } from "./";

export function newApiClient({ request }: {
  request: Request
}) {
  const response = new Response();
  const query = new QueryClient();
  const client = new ApiClient(AGGY_BASE_URL, query);
  const responseBuidler = new ResponseBuidler(response, client);
  return {
    response: responseBuidler,
    client,
  } as const;
}

class ResponseBuidler {
  constructor(
    public response = new Response(),
    public client: ApiClient,
  ) { }

  ok<T>(data: T) {
    return json<T>(
      {
        ...data,
        dehydratedState: dehydrate(this.client.query),
      },
      { headers: this.response.headers, }
    )
  };

  redirect(to: string) {
    return redirect(to,
      { headers: this.response.headers, }
    );
  }

  badRequest<T>(data: T) {
    return json<T>(
      data, { status: 400, headers: this.response.headers, }
    )
  };

  serverError<T>(data: T) {
    return json<T>(
      data, { status: 500, headers: this.response.headers, }
    )
  };
}

