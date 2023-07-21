import * as zod from "zod";

export type AggyPost = {
  id: string,
  title: string,
  link: string,
  commentCount: number;
  epigram: Epigram,
}

export type Epigram = {
  id: string,
  ts: number;
  content: string,
  content_mime: string,
  author: {
    pkey: string,
    alias: string,
  }
  topFaces: Record<string, FaceSummary>,
  children: Epigram[],
}

export type FaceSummary = {
  count: number;
  userFacedAt?: number;
}

export const MIN_LENGTH_USERNAME = 5;
export const MAX_LENGTH_USERNAME = 32;

export const MIN_LENGTH_PASSWORD = 8;
export const MAX_LENGTH_PASSWORD = 1023;

export const validators = {
  createUserInput: zod.object({
    username: zod.string().min(MIN_LENGTH_USERNAME).max(MAX_LENGTH_USERNAME),
    email: zod.string().email(),
    password: zod.string().min(MIN_LENGTH_PASSWORD).max(MAX_LENGTH_PASSWORD),
  }),

  user: zod.object({
    createdAt: zod.number(),
    updatedAt: zod.number(),
    id: zod.string(),
    username: zod.string(),
    picUrl: zod.string().nullish(),
  }),

  createSessionInput: zod.object({
    userId: zod.string().nullish(),
    ipAddr: zod.string().ip(),
    userAgent: zod.string(),
  }),

  session: zod.object({
    createdAt: zod.number(),
    updatedAt: zod.number(),
    id: zod.string(),
    userId: zod.string().nullish(),
    ipAddr: zod.string(),
    userAgent: zod.string(),
    expiresAt: zod.number(),
  }),
}

export type User = typeof validators.user._type;
export type Session = typeof validators.session._type;

export type CreateUserInput = typeof validators.createUserInput._type;
export type CreateSessionInput = typeof validators.createSessionInput._type;
