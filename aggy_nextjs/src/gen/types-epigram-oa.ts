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

export const CreateGram_Body = z
  .object({
    content: z.string().min(1),
    coty: z.string(),
    parentId: z.string().nullish(),
    authorPubkey: z.string(),
    createdAt: z.string().datetime({ offset: true }),
    id: z.string(),
    sig: z.string(),
    authorAlias: z.string().nullish(),
  })
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
export const CreateGramError = z.discriminatedUnion("error", [
  z
    .object({ id: z.string(), error: z.literal("parentNotFound") })
    .passthrough(),
  z
    .object({ issues: ValidationErrors, error: z.literal("invalidInput") })
    .passthrough(),
  z.object({ message: z.string(), error: z.literal("internal") }).passthrough(),
]);
export const GetGramError = z.discriminatedUnion("error", [
  z.object({ id: z.string(), error: z.literal("notFound") }).passthrough(),
  z.object({ message: z.string(), error: z.literal("internal") }).passthrough(),
]);

export const endpoints = {
  CreateGram: {
    method: "post",
    path: "/epigram/grams",
    parameters: {
      body: {
        name: "body",
        type: "Body",
        schema: CreateGram_Body,
      },
    },
    response: Gram,
    errors: [
      {
        status: 400,
        description: `Invalid input`,
        schema: CreateGramError,
      },
      {
        status: 404,
        description: `Parent Not Found`,
        schema: CreateGramError,
      },
      {
        status: 500,
        description: `Internal server error`,
        schema: CreateGramError,
      },
    ],
  },
  GetGram: {
    method: "get",
    path: "/epigram/grams/:id",
    parameters: {
      includeReplies: {
        name: "includeReplies",
        type: "Query",
        schema: z.boolean().optional(),
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
        status: 404,
        description: `Not Found`,
        schema: GetGramError,
      },
      {
        status: 500,
        description: `Internal server error`,
        schema: GetGramError,
      },
    ],
  },
};
