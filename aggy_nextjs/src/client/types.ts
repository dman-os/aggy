import * as zod from "zod";
import * as oaAggy from "@/gen/types-aggy-oa"
import * as oaEpi from "@/gen/types-epigram-oa"
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

export const MIN_LENGTH_USERNAME = assertNotNull(oaAggy.CreateUser_Body.shape.username.minLength);
export const MAX_LENGTH_USERNAME = assertNotNull(oaAggy.CreateUser_Body.shape.username.maxLength);

export const MIN_LENGTH_PASSWORD = assertNotNull(oaAggy.CreateUser_Body.shape.password.minLength);
export const MAX_LENGTH_PASSWORD = assertNotNull(oaAggy.CreateUser_Body.shape.password.maxLength);

export const MAX_LENGTH_TITLE = assertNotNull(oaAggy.CreatePost_Body.shape.title.maxLength);

export const validators = {
  createUserBody: oaAggy.CreateUser_Body.merge(
    zod.object({
      email: zod.string().email().nullish(),
    })
  ),
  user: oaAggy.User.merge(
    zod.object({
      id: zod.string(),
    })
  ),
  post: oaAggy.Post.merge(
    zod.object({
    })
  ),
  gram: oaEpi.Gram,

  createSessionBody: oaAggy.CreateWebSession_Body.merge(
    zod.object({
      ipAddr: zod.string().ip(),
    })
  ),
  updateSessionBody: oaAggy.endpoints.UpdateWebSession.parameters.body.schema.merge(
    zod.object({
      authSessionId: zod.string().nullish(),
    })
  ),
  session: oaAggy.Session.merge(
    zod.object({
      id: zod.string(),
      userId: zod.string().nullish(),
    })
  ),

  authenticateBody: oaAggy.Authenticate_Body,
  authenticateResponse: oaAggy.endpoints.Authenticate.response.merge(
    zod.object({
      sessionId: zod.string(),
      userId: zod.string(),
    })
  ),

  listPostsQuery: zod.object({
    limit: oaAggy.endpoints.ListPosts.parameters.limit.schema,
    afterCursor: oaAggy.endpoints.ListPosts.parameters.afterCursor.schema,
    beforeCursor: oaAggy.endpoints.ListPosts.parameters.beforeCursor.schema,
    filter: oaAggy.endpoints.ListPosts.parameters.filter.schema,
    sortingField: oaAggy.endpoints.ListPosts.parameters.sortingField.schema,
    sortingOrder: oaAggy.endpoints.ListPosts.parameters.sortingOrder.schema,
  }),
  listPostsResponse: oaAggy.endpoints.ListPosts.response,
  createPostBody: oaAggy.CreatePost_Body.merge(
    zod.object({
      url: zod.string().url().nullish()
    })
  ).refine(obj => obj.url || obj.body, {
    message: "Either url or body must be present."
  }),
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
export type CreatePostBody = zod.infer<typeof validators.createPostBody>;
