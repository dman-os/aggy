
"use server"
import { NextResponse } from 'next/server'
import * as zod from "zod";
import { fromZodError } from "zod-validation-error";

import {
  CreateUserInput, AggyApiError,
} from "@/client";
import { dbg } from "@/utils";
import { apiClient } from '@/client/index.server';


export async function register(data: FormData) {
  dbg({ data: [...data.entries()] });
  const { client } = apiClient();
  try {
    const input: Partial<CreateUserInput> = {
      email: data.get("email")?.toString(),
      password: data.get("password")?.toString(),
      username: data.get("username")?.toString(),
    };
    const { id } = await client.aggy.register(input);
    dbg({ id });
    NextResponse.redirect(data.get("redirectTo")?.toString() ?? "/");
  } catch (err) {
    dbg({ err });
    if (err instanceof zod.ZodError) {
      const zodErr = err as zod.ZodError<CreateUserInput>;
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
}
