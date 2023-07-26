import { dbg } from "@/utils";
import * as T from "./types";
import * as zod from "zod";
import { fromZodError } from "zod-validation-error";

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
          "Content-Type": "application/json",
          "Authorization": `Bearer ${this.serviceSecret}`,
        },
      }
    );
    if (!response.ok) {
      throw await AggyApiError.fromResponse(response);
    }
    const body = await response.json();
    return zodMustParse(T.validators.session, body);
  }
}

export class AggyApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    public bodyJson: object,
  ) {
    super(`AggyApiError: ${status} - ${code}`)
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

function zodMustParse<I, O>(schema: zod.Schema<O>, input: I) {
  const result = schema.safeParse(input)
  if (!result.success) {
    throw Error(`Unexpected error validating schema: ${fromZodError(result.error).toString()}`);
  }
  return result.data;
}
