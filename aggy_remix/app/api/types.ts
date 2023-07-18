
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
