import * as zod from "zod";
import * as oa from "@/gen/types-aggy-oa"
import { assertNotNull } from "@/utils";

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

export const MIN_LENGTH_USERNAME = assertNotNull(oa.CreateUser_Body.shape.username.minLength);
export const MAX_LENGTH_USERNAME = assertNotNull(oa.CreateUser_Body.shape.username.maxLength);

export const MIN_LENGTH_PASSWORD = assertNotNull(oa.CreateUser_Body.shape.password.minLength);
export const MAX_LENGTH_PASSWORD = assertNotNull(oa.CreateUser_Body.shape.password.maxLength);

export const validators = {
  createUserBody: oa.CreateUser_Body.merge(
    zod.object({
      email: zod.string().email().nullish(),
    })
  ),
  user: oa.User.merge(
    zod.object({
      id: zod.string(),
    })
  ),

  createSessionBody: oa.CreateWebSession_Body.merge(
    zod.object({
      ipAddr: zod.string().ip(),
    })
  ),
  updateSessionBody: oa.endpoints.UpdateWebSession.parameters.body.schema.merge(
    zod.object({
      authSessionId: zod.string().nullish(),
    })
  ),
  session: oa.Session.merge(
    zod.object({
      id: zod.string(),
      userId: zod.string().nullish(),
    })
  ),

  authenticateBody: oa.Authenticate_Body,
  authenticateResponse: oa.endpoints.Authenticate.response.merge(
    zod.object({
      sessionId: zod.string(),
      userId: zod.string(),
    })
  ),
}

export type User = typeof validators.user._type;
export type Session = typeof validators.session._type;

export type CreateUserBody = typeof validators.createUserBody._type;
export type CreateSessionBody = typeof validators.createSessionBody._type;
export type AuthenticateBody = typeof validators.authenticateBody._type;
export type AuthenticateResponse = typeof validators.authenticateResponse._type;
export type UpdateSessionBody = typeof validators.updateSessionBody._type;
