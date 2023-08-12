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
  post: oa.Post.merge(
    zod.object({
    })
  ),
  gram: oa.Gram,

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

  listPostsQuery: zod.object({
    limit: oa.endpoints.ListPosts.parameters.limit.schema,
    afterCursor: oa.endpoints.ListPosts.parameters.afterCursor.schema,
    beforeCursor: oa.endpoints.ListPosts.parameters.beforeCursor.schema,
    filter: oa.endpoints.ListPosts.parameters.filter.schema,
    sortingField: oa.endpoints.ListPosts.parameters.sortingField.schema,
    sortingOrder: oa.endpoints.ListPosts.parameters.sortingOrder.schema,
  }),
  listPostsResponse: oa.endpoints.ListPosts.response
}

export type User = zod.infer<typeof validators.user>;
export type Session = zod.infer<typeof validators.session>;
export type Post = zod.infer<typeof validators.post>;
export type Gram = zod.infer<typeof validators.gram>;

export type CreateUserBody = zod.infer<typeof validators.createUserBody>;
export type CreateSessionBody = zod.infer<typeof validators.createSessionBody>;
export type AuthenticateBody = zod.infer<typeof validators.authenticateBody>;
export type AuthenticateResponse = zod.infer<typeof validators.authenticateResponse>;
export type UpdateSessionBody = zod.infer<typeof validators.updateSessionBody>;
export type ListPostsQuery = zod.infer<typeof validators.listPostsQuery>;
