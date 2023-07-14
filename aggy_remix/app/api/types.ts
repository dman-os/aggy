
export type AggyPost = {
  id: string,
  title: string,
  link: string,
  commentCount: number;
  epigram: Epigram,
}

export type Epigram = {
  id: string,
  body_html: string,
  author: {
    pkey: string,
    alias: string,
  }
  topFaces: Record<string, Face>,
}

export type Face = {
  count: number;
  userFacedAtTs?: number;
}
