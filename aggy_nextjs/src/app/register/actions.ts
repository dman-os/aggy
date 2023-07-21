
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
    const input: Partial<T.CreateUserInput> = {
      email: data.get("email")?.toString(),
      password: data.get("password")?.toString(),
      username: data.get("username")?.toString(),
    };
    const { id } = await client.aggy.register(input);
    dbg({ id });
  } catch (err) {
    if (err instanceof zod.ZodError) {
      const zodErr = err as zod.ZodError<T.CreateUserInput>;
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
    return {
      fieldErrors: "Server error",
    };
  }
  redirect(data.get("redirectTo")?.toString() ?? "/");
}
