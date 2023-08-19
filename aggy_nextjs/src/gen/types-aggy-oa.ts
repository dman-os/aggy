import { z } from "zod";

export type Gram = {
  id: string;
  createdAt: string;
  content: string;
  coty: string;
  parentId?: string | null;
  authorPubkey: string;
  authorAlias?: string | null;
  sig: string;
  replies?: Array<Gram> | null;
  replyCount?: number | null;
};
export type ValidationErrors = {};
export type ValidationError = {
  code: string;
  message?: string | null;
  params: {};
};
export type ValidationErrorsKind =
  | ValidationErrors
  | {}
  | Array<ValidationError>;

export const Authenticate_Body = z
  .object({ identifier: z.string(), password: z.string() })
  .passthrough();
export const AuthenticateError = z.discriminatedUnion("error", [
  z.object({ error: z.literal("credentialsRejected") }).passthrough(),
  z.object({ message: z.string(), error: z.literal("internal") }).passthrough(),
]);
export const Reply_Body = z
  .object({ parentId: z.string().nullish(), body: z.string().min(1) })
  .passthrough();
export const Gram: z.ZodType<Gram> = z.lazy(() =>
  z
    .object({
      id: z.string(),
      createdAt: z.string().datetime({ offset: true }),
      content: z.string(),
      coty: z.string(),
      parentId: z.string().nullish(),
      authorPubkey: z.string(),
      authorAlias: z.string().nullish(),
      sig: z.string(),
      replies: z.array(Gram).nullish(),
      replyCount: z.number().int().nullish(),
    })
    .passthrough()
);
export const ValidationError = z
  .object({
    code: z.string(),
    message: z.string().nullish(),
    params: z.record(z.object({}).partial().passthrough()),
  })
  .passthrough();
export const ValidationErrorsKind: z.ZodType<ValidationErrorsKind> = z.lazy(
  () =>
    z.union([
      ValidationErrors,
      z.record(ValidationErrors),
      z.array(ValidationError),
    ])
);
export const ValidationErrors: z.ZodType<ValidationErrors> = z.lazy(() =>
  z.record(ValidationErrorsKind)
);
export const ReplyError = z.discriminatedUnion("error", [
  z.object({ id: z.string(), error: z.literal("notFound") }).passthrough(),
  z.object({ error: z.literal("accessDenied") }).passthrough(),
  z
    .object({ issues: ValidationErrors, error: z.literal("invalidInput") })
    .passthrough(),
  z.object({ message: z.string(), error: z.literal("internal") }).passthrough(),
]);
export const PostSortingField = z.enum(["createdAt", "updatedAt"]);
export const sortingField = PostSortingField.nullish();
export const SortingOrder = z.enum(["ascending", "descending"]);
export const sortingOrder = SortingOrder.nullish();
export const Post = z
  .object({
    id: z.string().uuid(),
    createdAt: z.string().datetime({ offset: true }),
    updatedAt: z.string().datetime({ offset: true }),
    epigramId: z.string(),
    title: z.string(),
    url: z.string().nullish(),
    body: z.string().nullish(),
    authorUsername: z.string(),
    authorPicUrl: z.string().nullish(),
    authorPubKey: z.string(),
    epigram: Gram.nullish(),
  })
  .passthrough();
export const ListPostsError = z.discriminatedUnion("error", [
  z
    .object({ issues: ValidationErrors, error: z.literal("invalidInput") })
    .passthrough(),
  z.object({ message: z.string(), error: z.literal("internal") }).passthrough(),
]);
export const CreatePost_Body = z
  .object({
    title: z.string().min(1).max(80),
    url: z.string().nullish(),
    body: z.string().min(1).nullish(),
  })
  .passthrough();
export const CreatePostError = z.discriminatedUnion("error", [
  z.object({ error: z.literal("accessDenied") }).passthrough(),
  z
    .object({ issues: ValidationErrors, error: z.literal("invalidInput") })
    .passthrough(),
  z.object({ message: z.string(), error: z.literal("internal") }).passthrough(),
]);
export const GetPostError = z.discriminatedUnion("error", [
  z
    .object({ id: z.string().uuid(), error: z.literal("notFound") })
    .passthrough(),
  z.object({ message: z.string(), error: z.literal("internal") }).passthrough(),
]);
export const UserSortingField = z.enum([
  "username",
  "email",
  "createdAt",
  "updatedAt",
]);
export const ListUsersRequest = z
  .object({
    limit: z.number().int().gte(1).lte(100).nullable(),
    afterCursor: z.string().nullable(),
    beforeCursor: z.string().nullable(),
    filter: z.string().nullable(),
    sortingField: UserSortingField.nullable(),
    sortingOrder: SortingOrder.nullable(),
  })
  .partial()
  .passthrough();
export const User = z
  .object({
    id: z.string().uuid(),
    createdAt: z.string().datetime({ offset: true }),
    updatedAt: z.string().datetime({ offset: true }),
    email: z.string().nullish(),
    username: z.string(),
    picUrl: z.string().nullish(),
    pubKey: z.string(),
  })
  .passthrough();
export const ListUsersResponse = z
  .object({ cursor: z.string().nullish(), items: z.array(User) })
  .passthrough();
export const ListUsersError = z.discriminatedUnion("error", [
  z.object({ error: z.literal("accessDenied") }).passthrough(),
  z
    .object({ issues: ValidationErrors, error: z.literal("invalidInput") })
    .passthrough(),
  z.object({ message: z.string(), error: z.literal("internal") }).passthrough(),
]);
export const CreateUser_Body = z
  .object({
    username: z
      .string()
      .min(5)
      .max(32)
      .regex(/^[a-zA-Z0-9]+([_-]?[a-zA-Z0-9])*$/),
    email: z.string().nullish(),
    password: z.string().min(8).max(1024),
  })
  .passthrough();
export const CreateUserError = z.discriminatedUnion("error", [
  z
    .object({ username: z.string(), error: z.literal("usernameOccupied") })
    .passthrough(),
  z
    .object({ email: z.string(), error: z.literal("emailOccupied") })
    .passthrough(),
  z
    .object({ issues: ValidationErrors, error: z.literal("invalidInput") })
    .passthrough(),
  z.object({ message: z.string(), error: z.literal("internal") }).passthrough(),
]);
export const GetUserError = z.discriminatedUnion("error", [
  z
    .object({ id: z.string().uuid(), error: z.literal("notFound") })
    .passthrough(),
  z.object({ error: z.literal("accessDenied") }).passthrough(),
  z.object({ message: z.string(), error: z.literal("internal") }).passthrough(),
]);
export const UpdateUser_Body = z
  .object({
    username: z
      .string()
      .min(5)
      .max(32)
      .regex(/^[a-zA-Z0-9]+([_-]?[a-zA-Z0-9])*$/)
      .nullable(),
    email: z.string().nullable(),
    picUrl: z.string().nullable(),
    password: z.string().min(8).max(1024).nullable(),
  })
  .partial()
  .passthrough();
export const UpdateUserError = z.discriminatedUnion("error", [
  z
    .object({ id: z.string().uuid(), error: z.literal("notFound") })
    .passthrough(),
  z.object({ error: z.literal("accessDenied") }).passthrough(),
  z
    .object({ username: z.string(), error: z.literal("usernameOccupied") })
    .passthrough(),
  z
    .object({ email: z.string(), error: z.literal("emailOccupied") })
    .passthrough(),
  z
    .object({ issues: ValidationErrors, error: z.literal("invalidInput") })
    .passthrough(),
  z.object({ message: z.string(), error: z.literal("internal") }).passthrough(),
]);
export const std_net_IpAddr = z.string();
export const CreateWebSession_Body = z
  .object({
    ipAddr: std_net_IpAddr,
    authSessionId: z.string().uuid().nullish(),
    userAgent: z.string(),
  })
  .passthrough();
export const Session = z
  .object({
    id: z.string().uuid(),
    ipAddr: std_net_IpAddr,
    userAgent: z.string(),
    expiresAt: z.string().datetime({ offset: true }),
    createdAt: z.string().datetime({ offset: true }),
    updatedAt: z.string().datetime({ offset: true }),
    userId: z.string().uuid().nullish(),
    token: z.string().nullish(),
    tokenExpiresAt: z.string().datetime({ offset: true }).nullish(),
  })
  .passthrough();
export const CreateWebSessionError = z.discriminatedUnion("error", [
  z.object({ error: z.literal("accessDenied") }).passthrough(),
  z
    .object({ id: z.string().uuid(), error: z.literal("authSessionNotFound") })
    .passthrough(),
  z.object({ message: z.string(), error: z.literal("internal") }).passthrough(),
]);
export const GetWebSessionError = z.discriminatedUnion("error", [
  z.object({ error: z.literal("accessDenied") }).passthrough(),
  z
    .object({ id: z.string().uuid(), error: z.literal("notFound") })
    .passthrough(),
  z.object({ message: z.string(), error: z.literal("internal") }).passthrough(),
]);
export const UpdateWebSessionError = z.discriminatedUnion("error", [
  z.object({ error: z.literal("accessDenied") }).passthrough(),
  z
    .object({ id: z.string().uuid(), error: z.literal("notFound") })
    .passthrough(),
  z
    .object({ id: z.string().uuid(), error: z.literal("authSessionNotFound") })
    .passthrough(),
  z.object({ message: z.string(), error: z.literal("internal") }).passthrough(),
]);

export const endpoints = {
  Authenticate: {
    method: "post",
    path: "/aggy/authenticate",
    parameters: {
      body: {
        name: "body",
        type: "Body",
        schema: Authenticate_Body,
      },
    },
    response: z
      .object({
        sessionId: z.string().uuid(),
        userId: z.string().uuid(),
        token: z.string(),
        expiresAt: z.string().datetime({ offset: true }),
      })
      .passthrough(),
    errors: [
      {
        status: 400,
        description: `Credentials rejected`,
        schema: AuthenticateError,
      },
      {
        status: 500,
        description: `Internal server error`,
        schema: AuthenticateError,
      },
    ],
  },
  Reply: {
    method: "post",
    path: "/aggy/grams/:id/replies",
    parameters: {
      body: {
        name: "body",
        type: "Body",
        schema: Reply_Body,
      },
      id: {
        name: "id",
        type: "Path",
        schema: z.string(),
      },
    },
    response: Gram,
    errors: [
      {
        status: 400,
        description: `Invalid input`,
        schema: ReplyError,
      },
      {
        status: 401,
        description: `Access Denied`,
        schema: ReplyError,
      },
      {
        status: 404,
        description: `Not Found`,
        schema: ReplyError,
      },
      {
        status: 500,
        description: `Internal server error`,
        schema: ReplyError,
      },
    ],
  },
  ListPosts: {
    method: "get",
    path: "/aggy/posts",
    parameters: {
      authToken: {
        name: "authToken",
        type: "Query",
        schema: z.string().nullish(),
      },
      limit: {
        name: "limit",
        type: "Query",
        schema: z.number().int().gte(1).lte(100).nullish(),
      },
      afterCursor: {
        name: "afterCursor",
        type: "Query",
        schema: z.string().nullish(),
      },
      beforeCursor: {
        name: "beforeCursor",
        type: "Query",
        schema: z.string().nullish(),
      },
      filter: {
        name: "filter",
        type: "Query",
        schema: z.string().nullish(),
      },
      sortingField: {
        name: "sortingField",
        type: "Query",
        schema: sortingField,
      },
      sortingOrder: {
        name: "sortingOrder",
        type: "Query",
        schema: sortingOrder,
      },
    },
    response: z
      .object({ cursor: z.string().nullish(), items: z.array(Post) })
      .passthrough(),
    errors: [
      {
        status: 400,
        description: `Invalid input`,
        schema: ListPostsError,
      },
      {
        status: 500,
        description: `Internal server error`,
        schema: ListPostsError,
      },
    ],
  },
  CreatePost: {
    method: "post",
    path: "/aggy/posts",
    parameters: {
      body: {
        name: "body",
        type: "Body",
        schema: CreatePost_Body,
      },
    },
    response: Post,
    errors: [
      {
        status: 400,
        description: `Invalid input`,
        schema: CreatePostError,
      },
      {
        status: 401,
        description: `Access Denied`,
        schema: CreatePostError,
      },
      {
        status: 500,
        description: `Internal server error`,
        schema: CreatePostError,
      },
    ],
  },
  GetPost: {
    method: "get",
    path: "/aggy/posts/:id",
    parameters: {
      includeReplies: {
        name: "includeReplies",
        type: "Query",
        schema: z.boolean().optional(),
      },
      id: {
        name: "id",
        type: "Path",
        schema: z.string().uuid(),
      },
    },
    response: Post,
    errors: [
      {
        status: 404,
        description: `Not Found`,
        schema: GetPostError,
      },
      {
        status: 500,
        description: `Internal server error`,
        schema: GetPostError,
      },
    ],
  },
  ListUsers: {
    method: "get",
    path: "/aggy/users",
    parameters: {
      body: {
        name: "body",
        type: "Body",
        schema: ListUsersRequest,
      },
    },
    response: ListUsersResponse,
    errors: [
      {
        status: 400,
        description: `Invalid input`,
        schema: ListUsersError,
      },
      {
        status: 401,
        description: `Access denied`,
        schema: ListUsersError,
      },
      {
        status: 500,
        description: `Internal server error`,
        schema: ListUsersError,
      },
    ],
  },
  CreateUser: {
    method: "post",
    path: "/aggy/users",
    parameters: {
      body: {
        name: "body",
        type: "Body",
        schema: CreateUser_Body,
      },
    },
    response: User,
    errors: [
      {
        status: 400,
        description: `Username occupied | Invalid input | Email occupied`,
        schema: CreateUserError,
      },
      {
        status: 500,
        description: `Internal server error`,
        schema: CreateUserError,
      },
    ],
  },
  GetUser: {
    method: "get",
    path: "/aggy/users/:id",
    parameters: {
      id: {
        name: "id",
        type: "Path",
        schema: z.string().uuid(),
      },
    },
    response: User,
    errors: [
      {
        status: 401,
        description: `Access denied`,
        schema: GetUserError,
      },
      {
        status: 404,
        description: `Not found`,
        schema: GetUserError,
      },
      {
        status: 500,
        description: `Internal server error`,
        schema: GetUserError,
      },
    ],
  },
  UpdateUser: {
    method: "patch",
    path: "/aggy/users/:id",
    parameters: {
      body: {
        name: "body",
        type: "Body",
        schema: UpdateUser_Body,
      },
      id: {
        name: "id",
        type: "Path",
        schema: z.string().uuid(),
      },
    },
    response: User,
    errors: [
      {
        status: 400,
        description: `Username occupied | Email occupied | Invalid input`,
        schema: UpdateUserError,
      },
      {
        status: 401,
        description: `Access denied`,
        schema: UpdateUserError,
      },
      {
        status: 404,
        description: `Not found`,
        schema: UpdateUserError,
      },
      {
        status: 500,
        description: `Internal server error`,
        schema: UpdateUserError,
      },
    ],
  },
  CreateWebSession: {
    method: "post",
    path: "/aggy/web/sessions",
    parameters: {
      body: {
        name: "body",
        type: "Body",
        schema: CreateWebSession_Body,
      },
    },
    response: Session,
    errors: [
      {
        status: 401,
        description: `Access denied`,
        schema: CreateWebSessionError,
      },
      {
        status: 404,
        description: `Auth Session Not Found`,
        schema: CreateWebSessionError,
      },
      {
        status: 500,
        description: `Internal server error`,
        schema: CreateWebSessionError,
      },
    ],
  },
  GetWebSession: {
    method: "get",
    path: "/aggy/web/sessions/:id",
    parameters: {
      id: {
        name: "id",
        type: "Path",
        schema: z.string().uuid(),
      },
    },
    response: Session,
    errors: [
      {
        status: 401,
        description: `Access denied`,
        schema: GetWebSessionError,
      },
      {
        status: 404,
        description: `Not Found`,
        schema: GetWebSessionError,
      },
      {
        status: 500,
        description: `Internal server error`,
        schema: GetWebSessionError,
      },
    ],
  },
  UpdateWebSession: {
    method: "patch",
    path: "/aggy/web/sessions/:id",
    parameters: {
      body: {
        name: "body",
        type: "Body",
        schema: z
          .object({ authSessionId: z.string().uuid().nullable() })
          .partial()
          .passthrough(),
      },
      id: {
        name: "id",
        type: "Path",
        schema: z.string().uuid(),
      },
    },
    response: Session,
    errors: [
      {
        status: 401,
        description: `Access denied`,
        schema: UpdateWebSessionError,
      },
      {
        status: 404,
        description: `Auth Session Not Found | Not Found`,
        schema: UpdateWebSessionError,
      },
      {
        status: 500,
        description: `Internal server error`,
        schema: UpdateWebSessionError,
      },
    ],
  },
};
