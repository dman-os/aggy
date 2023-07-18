import { AggyClient } from "./aggy";
import type { QueryClient } from "@tanstack/react-query";
import type * as T from "./types";
import { queryFamily } from "~/utils/atoms";
import * as zod from "zod";

export * from "./aggy";
// import { queryFamily } from "~/utils/atoms";

export class ApiClient {
  constructor(
    public aggyBaseUrl: string,
    public query: QueryClient,
  ) {
  }

  async getTopPosts() {
    return topPosts;
  }
}

export enum QUERY_KEYS {
  POSTS = 'POSTS',
  POST = 'POST',
}

type DePromisify<T> = T extends Promise<infer Inner> ? Inner : T;
type FnProps<T> = ({ [P in keyof T]: T[P] extends Function ? P : never })[keyof T];
type ApiResponse<FnName extends FnProps<ApiClient>> = DePromisify<ReturnType<ApiClient[FnName]>>;

export const getTopPosts = queryFamily<void, ApiResponse<"getTopPosts">>(
  (client) => ({
    queryKey: [QUERY_KEYS.POSTS],
    queryFn: async () => client.getTopPosts(),
    // queryFn: async () => products,
  })
);


const topPostsSeed = [
  {
    title: "DIY RISC-V in the bomblands.",
    link: "https://news.ycombinator.com",
    commentCount: 15,
    epigram: {
      author: {
        pkey: "dadyoyo1",
        alias: "bigyoyo"
      },
      topFaces: {
        "b": { count: 12314, ts: Date.now() },
        "(\\\"_\")=b": { count: 1 },
      } as Record<string, T.FaceSummary>
    },
    children: [
      {
        content: '<p>This is a comment</p>',
        author: {
          pkey: "dadyoyo1",
          alias: "bigyoyo"
        },
        topFaces: {
          "b": { count: 12314, ts: Date.now() },
          "(\\\"_\")=b": { count: 1 },
        } as Record<string, T.FaceSummary>,
        children: [
          {
            content: '<p>This is also a comment</p>',
            author: {
              pkey: "dadyoyo1",
              alias: "bigyoyo"
            },
            topFaces: {
              "b": { count: 12314, ts: Date.now() },
              "(\\\"_\")=b": { count: 1 },
            } as Record<string, T.FaceSummary>
          }
        ] as T.Epigram[]
      }
    ]
  },
  {
    title: "Apple is in talks to buy Capitol Hill",
    commentCount: 15,
    epigram: {
      author: {
        pkey: "lib22",
        alias: "freespeech112"
      },
      topFaces: {
        "p": { count: 123 },
        ":'<": { count: 1 },
      }
    },
  },
  {
    title: "Yo, I'm tha admin yo",
    commentCount: 15,
    epigram: {
      author: {
        pkey: "admin1",
        alias: "dang"
      },
      topFaces: {
        "b": { count: 123 }
      } as Record<string, T.FaceSummary>
    },
  },
];
export const topPosts: T.AggyPost[] = [
  ...topPostsSeed,
  ...topPostsSeed,
  ...topPostsSeed,
].reduce<T.AggyPost[]>(
  (arr, seed, idx) => {
    const id = `post_${idx}`;
    const epigramId = `epigram_${idx}`;
    const commentsLink = `p/${id}`
    const link = seed.link ?? commentsLink;
    let commentIdx = 0;
    function seedToEpigram(seed: Partial<T.Epigram>): T.Epigram {
      return {
        ...seed,
        content_mime: 'text/html',
        ts: Date.now() - Math.random() * 111222,
        id: `epigram_${commentIdx++ ^ idx}`,
        children: (seed.children ?? []).map(seedToEpigram),
      } as T.Epigram;
    }
    arr.push({
      id,
      link,
      ...seed,
      epigram: {
        id: epigramId,
        ts: Date.now() - Math.random() * 111222,
        content: `<a href="${link}">${seed.title}</a>`,
        content_mime: 'text/html',
        ...seed.epigram,
        children: (seed.children ?? []).map(seedToEpigram)
      },
    });
    return arr;
  }
  , []
);
