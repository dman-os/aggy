import type * as T from "./types";
import * as zod from "zod";
import { fromZodError } from "zod-validation-error";

export const MIN_LENGTH_USERNAME = 5;
export const MAX_LENGTH_USERNAME = 32;

export const MIN_LENGTH_PASSWORD = 8;
export const MAX_LENGTH_PASSWORD = 1023;

export const createUserInputValidator = zod.object({
  username: zod.string().min(MIN_LENGTH_USERNAME).max(MAX_LENGTH_USERNAME),
  email: zod.string().email(),
  password: zod.string().min(MIN_LENGTH_PASSWORD).max(MAX_LENGTH_PASSWORD),
})
export type CreateUserInput = typeof createUserInputValidator._type;

const createUserResponseValidator = zod.object({
  createdAt: zod.number(),
  updatedAt: zod.number(),
  id: zod.string(),
  username: zod.string(),
  picUrl: zod.string().nullish(),
});

export class AggyClient {
  constructor(
    public baseUrl: string
  ) { }

  async register(uncleanInput: any) {
    const input = createUserInputValidator.parse(uncleanInput);
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
    return zodMustParse(createUserResponseValidator, body);

  }
  async login() {
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
