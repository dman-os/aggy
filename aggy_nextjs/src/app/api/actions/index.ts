"use server"

import { redirect } from "next/navigation";
import * as zod from "zod";
import { fromZodError } from "zod-validation-error";

import { apiClient } from "@/client/index.server";
import { dbg } from "@/utils";
import { AggyApiError, T } from "@/client";

export async function doface(data: FormData) {
  dbg({ data: [...data.entries()] });
}

export async function unface(data: FormData) {
  dbg({ data: [...data.entries()] });
}

export async function comment(data: FormData) {
  dbg({ data: [...data.entries()] });
}

export async function login(data: FormData) {
  dbg({ data: [...data.entries()] });
}

export async function reply(data: FormData) {
  const { client, session } = apiClient();
  const { token } = await session.load();
  const targetGramId = data.get("parentId")!.toString();
  if (!token) {
    redirect(`/login?redirectTo=/g/${targetGramId}`);
  }
  let redirectGramId;
  try {
    const { parentId } = await client.aggy.reply(
      targetGramId,
      {
        body: data.get("body")!.toString(),
      },
      token
    );
    redirectGramId = parentId;
  } catch (err) {
    if (err instanceof zod.ZodError) {
      const zodErr = err as zod.ZodError<T.ReplyBody>;
      return {
        fieldErrors: zodErr.format(),
        formError: fromZodError(zodErr).toString(),
      };
    }
    if (err instanceof AggyApiError) {
      const aggyErr = err as AggyApiError;
      if (aggyErr.code === "notFound") {
        return {
          formError: "Post is no longer availaible.",
        }
      }
      if (aggyErr.code === "accessDenied") {
        return {
          formError: "Access denied.",
        }
      }
    }
    console.error({
      err,
    })
    return {
      formError: `Server error`,
    };
  }
  redirect(`/g/${redirectGramId}`);
}
