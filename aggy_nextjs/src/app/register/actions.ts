
"use server"
import { redirect } from 'next/navigation';
import * as zod from "zod";
import { fromZodError } from "zod-validation-error";

import {
  T, AggyApiError,
} from "@/client";
import { dbg } from "@/utils";
import { apiClient } from '@/client/index.server';


export async function register(data: FormData) {
  const { client } = apiClient();
  try {
    const { id } = await client.aggy.register({
      email: data.get("email")!.toString(),
      password: data.get("password")!.toString(),
      username: data.get("username")!.toString(),
    });
    dbg({ id });
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
      if (aggyErr.code === "usernameOccupied") {
        return {
          formError: "Username is already in use",
        }
      }
      if (aggyErr.code === "emailOccupied") {
        return {
          formError: "Email is already in use",
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
