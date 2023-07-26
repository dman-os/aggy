import { z } from "zod";

export type ValidationErrors = {};
export type ValidationError = {
  code: string;
  message?: string | undefined;
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
export const UserSortingField = z.enum([
  "username",
  "email",
  "createdAt",
  "updatedAt",
]);
export const SortingOrder = z.enum(["ascending", "descending"]);
export const ListUsersRequest = z
  .object({
    limit: z.number().int().gte(0).nullable(),
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
export const DeleteUserError = z.discriminatedUnion("error", [
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
        description: `Username occupied | Email occupied | Invalid input`,
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
  DeleteUser: {
    method: "delete",
    path: "/aggy/users/:id",
    parameters: {
      id: {
        name: "id",
        type: "Path",
        schema: z.string().uuid(),
      },
    },
    response: z.void(),
    errors: [
      {
        status: 401,
        description: `Access denied`,
        schema: DeleteUserError,
      },
      {
        status: 500,
        description: `Internal server error`,
        schema: DeleteUserError,
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
        description: `Invalid input | Email occupied | Username occupied`,
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
