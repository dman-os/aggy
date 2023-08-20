import { notFound } from "next/navigation";

import { GramDetails } from "@/app/_components";
import { getCsrfToken } from "@/utils/index.server";
import { apiClient } from "@/client/index.server";

export default async function GramDetailsPage(
  { params }: { params: { 'gram-id': string } }
) {
  const { client, session } = apiClient();
  const gram = await client.epigram.getGram(params["gram-id"]);
  if (!gram) {
    return notFound();
  }
  return <GramDetails gram={gram} csrfToken={getCsrfToken()} />
}
