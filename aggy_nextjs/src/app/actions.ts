"use server"
import { dbg } from "@/utils";

export async function logout(data: FormData) {
  dbg({ data: [...data.entries()] });
}
