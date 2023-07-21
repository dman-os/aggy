import { dbg } from "@/utils";
import * as T from "./types";
import * as zod from "zod";
import { fromZodError } from "zod-validation-error";

export class AggyClient {
  constructor(
    public serviceSecret: string,
    public baseUrl: string
  ) { }

  async register(uncleanInput: any) {
    const input = T.validators.createUserInput.parse(uncleanInput);
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
  async login() {
  }
  async createSession(uncleanInput: any) {
    const input = T.validators.createSessionInput.parse(uncleanInput);
    const response = await fetch(
      `${this.baseUrl}/web/sessions`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Authorization": `Bearer ${this.serviceSecret}`,
        },
        body: JSON.stringify(input)
      }
    );
    if (!response.ok) {
      throw dbg(await AggyApiError.fromResponse(response));
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
