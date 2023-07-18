"use server"

import { dbg } from "@/utils";

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
