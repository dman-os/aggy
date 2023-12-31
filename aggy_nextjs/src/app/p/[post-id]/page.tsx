import { notFound } from "next/navigation";

import { GramDetails } from "@/app/_components";
import { getCsrfToken } from "@/utils/index.server";
import { apiClient } from "@/client/index.server";

export default async function PostDetails(
  { params }: { params: { 'post-id': string } }
) {
  const { client, session } = apiClient();
  const post = await client.aggy.getPost(params["post-id"]);
  if (!post) {
    return notFound();
  }
  return <GramDetails gram={post.epigram!} csrfToken={getCsrfToken()} />
}
