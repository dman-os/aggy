import type * as T from "./types";

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
        "b": { count: 12314, userFacedAtTs: Date.now() },
        "(\\\"_\")=b": { count: 1 },
      } as Record<string, T.Face>
    },
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
      } as Record<string, T.Face>
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
    arr.push({
      id,
      link,
      ...seed,
      epigram: {
        id: epigramId,
        body_html: `<a href="${link}">${seed.title}</a>`,
        ...seed.epigram,
      },
    });
    return arr;
  }
  , []
);
