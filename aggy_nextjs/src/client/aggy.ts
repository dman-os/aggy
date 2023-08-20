import { dbg } from "@/utils";

import { zodMustParse } from "./";
import * as T from "./types";

export class AggyClient {
  constructor(
    public serviceSecret: string,
    public baseUrl: string
  ) { }

  async register(uncleanInput: T.CreateUserBody) {
    const input = T.validators.createUserBody.parse(uncleanInput);
    const response = await fetch(
      `${this.baseUrl}/users`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify(input)
      }
    );
    if (!response.ok) {
      throw await AggyApiError.fromResponse(response);
    }
    const body = await response.json();
    return zodMustParse(T.validators.user, body);
  }

  async authenticate(uncleanInput: T.AuthenticateBody) {
    const input = T.validators.authenticateBody.parse(uncleanInput);
    const response = await fetch(
      `${this.baseUrl}/authenticate`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify(input)
      }
    );
    if (!response.ok) {
      throw await AggyApiError.fromResponse(response);
    }
    const body = await response.json();
    return zodMustParse(T.validators.authenticateResponse, body);
  }

  async createSession(uncleanInput: T.CreateSessionBody) {
    const input = T.validators.createSessionBody.parse(uncleanInput);
    const response = await fetch(
      `${this.baseUrl}/web/sessions`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Authorization": `Bearer ${this.serviceSecret}`,
        },
        body: JSON.stringify(input),
      }
    );
    if (!response.ok) {
      throw await AggyApiError.fromResponse(response);
    }
    const body = await response.json();
    return zodMustParse(T.validators.session, body);
  }

  async updateSession(id: string, uncleanInput: T.UpdateSessionBody) {
    const input = T.validators.updateSessionBody.parse(uncleanInput);
    const response = await fetch(
      `${this.baseUrl}/web/sessions/${id}`,
      {
        method: "PATCH",
        headers: {
          "Content-Type": "application/json",
          "Authorization": `Bearer ${this.serviceSecret}`,
        },
        body: JSON.stringify(input),
      }
    );
    if (!response.ok) {
      throw await AggyApiError.fromResponse(response);
    }
    const body = await response.json();
    return zodMustParse(T.validators.session, body);
  }

  async getSession(id: string) {
    const response = await fetch(
      `${this.baseUrl}/web/sessions/${id}`,
      {
        method: "GET",
        headers: {
          "Authorization": `Bearer ${this.serviceSecret}`,
        },
        // cache: "only-if-cached"
      }
    );
    if (!response.ok) {
      throw await AggyApiError.fromResponse(response);
    }
    const body = await response.json();
    return zodMustParse(T.validators.session, body);
  }

  async listPosts(uncleanInput: T.ListPostsQuery) {
    const input = T.validators.listPostsQuery.parse(uncleanInput);
    const url = new URL(`${this.baseUrl}/posts`);
    for (const key in input) {
      const typedKey = key as keyof T.ListPostsQuery;
      if (input.hasOwnProperty(key) && input[typedKey]) {
        url.searchParams.set(key, input[typedKey]!.toString());
      }
    }
    const response = await fetch(
      url,
      {
        method: "GET",
        headers: {
          // "Content-Type": "application/json",
          "Authorization": `Bearer ${this.serviceSecret}`,
        },
        cache: 'no-cache'
        // body: JSON.stringify(input),
      }
    );
    if (!response.ok) {
      throw await AggyApiError.fromResponse(response);
    }
    const body = await response.json();
    return zodMustParse(T.validators.listPostsResponse, body);
  }

  async getPost(id: String) {
    const response = await fetch(
      `${this.baseUrl}/posts/${id}?includeReplies=true`,
      {
        method: "GET",
        headers: {
          // "Content-Type": "application/json",
          "Authorization": `Bearer ${this.serviceSecret}`,
        },
        // body: JSON.stringify(input),
      }
    );
    if (!response.ok) {
      const err = await AggyApiError.fromResponse(response);
      if (err instanceof AggyApiError && err.status === 404 && err.code == "notFound") {
        return undefined;
      }
      throw err;
    }
    const body = await response.json();
    return zodMustParse(T.validators.post, body);
  }

  async createPost(uncleanInput: T.CreatePostBody, authToken: string,) {
    const input = dbg(T.validators.createPostBody.parse(uncleanInput));
    const response = await fetch(
      `${this.baseUrl}/posts`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Authorization": `Bearer ${authToken}`
        },
        body: JSON.stringify(input)
      }
    );
    if (!response.ok) {
      throw await AggyApiError.fromResponse(response);
    }
    const body = await response.json();
    return zodMustParse(T.validators.post, body);
  }

  async reply(parentGramId: string, uncleanInput: T.ReplyBody, authToken: string,) {
    const input = T.validators.replyBody.parse(uncleanInput);
    const response = await fetch(
      `${this.baseUrl}/grams/${parentGramId}/replies`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Authorization": `Bearer ${authToken}`
        },
        body: JSON.stringify(input)
      }
    );
    if (!response.ok) {
      throw await AggyApiError.fromResponse(response);
    }
    const body = await response.json();
    return zodMustParse(T.validators.gram, body);
  }
}

export class AggyApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    public bodyJson: object,
  ) {
    super(`AggyApiError: ${status} - ${code}` + (bodyJson as any)["message"] ?? "")
  }

  static async fromResponse(response: Response) {
    if (
      [
        response.headers.get("Content-Type"),
        response.headers.get("content-type"),
      ].some(header => header && /^application\//.test(header) && /json/g.test(header))
    ) {
      const bodyJson = await response.json();
      return new AggyApiError(
        response.status,
        bodyJson["error"] ?? "",
        bodyJson
      );
    }
    return {
      status: response.status,
      message: await response.text()
    };
  }
}
