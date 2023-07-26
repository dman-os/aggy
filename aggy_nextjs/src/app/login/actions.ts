"use server"
import { redirect } from 'next/navigation';
import * as zod from "zod";
import { fromZodError } from "zod-validation-error";

import {
  T, AggyApiError,
} from "@/client";
import { apiClient } from '@/client/index.server';


export async function login(data: FormData) {
  const { client, session } = apiClient();
  const webSessionId = await session.id();
  try {
    const { sessionId: authSessionId } = await client.aggy.authenticate({
      identifier: data.get("username")!.toString(),
      password: data.get("password")!.toString(),
    });
    const { } = await client.aggy.updateSession(webSessionId, {
      authSessionId
    });
    // return {
    //   formError: `Success signing in as ${userId}`,
    // };
  } catch (err) {
    if (err instanceof zod.ZodError) {
      const zodErr = err as zod.ZodError<T.CreateUserBody>;
      return {
        fieldErrors: zodErr.format(),
        formError: fromZodError(zodErr).toString(),
      };
    }
    if (err instanceof AggyApiError) {
      const aggyErr = err as AggyApiError;
      if (aggyErr.code === "credentialsRejected") {
        return {
          formError: "Credentials rejected: username or password is wrong",
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
  redirect(data.get("redirectTo")?.toString() ?? "/");
}
