"use server"
import { redirect } from 'next/navigation';
import * as zod from "zod";
import { fromZodError } from "zod-validation-error";

import {
  T, AggyApiError,
} from "@/client";
import { dbg } from "@/utils";
import { apiClient } from '@/client/index.server';


export async function submitPost(data: FormData) {
  const { client, session } = apiClient();
  const { token } = await session.load();
  let postId;
  try {
    let body = data.get("body")?.toString();
    let url = data.get("url")?.toString();
    const { id } = await client.aggy.createPost({
      title: data.get("title")!.toString(),
      url: url !== '' ? url : undefined,
      body: body !== '' ? body : undefined,
    }, token!);
    postId = id;
  } catch (err) {
    if (err instanceof zod.ZodError) {
      const zodErr = err as zod.ZodError<T.CreatePostBody>;
      return {
        fieldErrors: zodErr.format(),
        formError: fromZodError(zodErr).toString(),
      };
    }
    /* if (err instanceof AggyApiError) {
      const aggyErr = err as AggyApiError;
    } */
    console.error({
      err,
    })
    return {
      formError: `Server error`,
    };
  }
  redirect(`/p/${postId}`)
}
