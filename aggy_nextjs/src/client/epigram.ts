import { dbg } from "@/utils";
import { zodMustParse } from "./";
import * as T from "./types";

export class EpigramClient {
  constructor(
    public serviceSecret: string,
    public baseUrl: string
  ) { }

  async getGram(id: String, includeReplies = true) {
    let url = new URL(`${this.baseUrl}/grams/${id}`);
    if (includeReplies) {
      url.searchParams.set("includeReplies", "true");
    }
    const response = await fetch(
      url,
      {
        method: "GET",
        headers: new Headers({
          // "Content-Type": "application/json",
          "Authorization": `Bearer ${this.serviceSecret}`,
        }),
        // body: JSON.stringify(input),
      }
    );
    if (!response.ok) {
      const err = await EpigramApiError.fromResponse(response);
      if (err instanceof EpigramApiError && err.status === 404 && err.code == "notFound") {
        return undefined;
      }
      throw err;
    }
    const body = await response.json();
    return zodMustParse(T.validators.gram, body);
  }
}

export class EpigramApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    public bodyJson: object,
  ) {
    super(`EpigramApiError: ${status} - ${code}` + (bodyJson as any)["message"] ?? "")
  }

  static async fromResponse(response: Response) {
    if (
      [
        response.headers.get("Content-Type"),
        response.headers.get("content-type"),
      ].some(header => header && /^application\//.test(header) && /json/g.test(header))
    ) {
      const bodyJson = await response.json();
      return new EpigramApiError(
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
