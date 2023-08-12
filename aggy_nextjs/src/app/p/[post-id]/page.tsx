import Link from "next/link";
import { notFound } from "next/navigation";

import { PostStatusLines } from "@/app/_components";
import * as T from "@/client/types";
import * as Actions from "@/app/api/actions"
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
  const epigram = post.epigram!;
  return <div className="flex flex-col gap-2">
    <div dangerouslySetInnerHTML={{ __html: epigram.content }}></div>
    <PostStatusLines post={post} csrfToken={getCsrfToken()} />
    <form action={Actions.comment} className="addCommentForm flex flex-col gap-1">
      <input type="hidden" name="csrf_token" value={getCsrfToken()} />
      <input name="parentId" value={epigram.id} type="hidden" />
      <textarea name="content" cols={80} rows={5} />
      <button type="submit" className="self-start p-1 b-1 rounded-2 b-outline hover:b-black dark:hover:b-white">
        add comment
      </button>
    </form>
    <br />
    <ul>{epigram.replies?.map(
      eg => <li key={eg.id}>
        <Comment gram={eg} csrfToken={getCsrfToken()} />
      </li>
    )}</ul>
  </div>
}

function Comment({ gram, csrfToken }: { gram: T.Gram, csrfToken: string, }) {
  return <>
    <div>
      <Link href={`/user/${gram.authorPubkey}`}>{gram.authorAlias ?? gram.authorPubkey}</Link>
      |
      {new Date(gram.createdAt).toLocaleString()}

      {Object.entries({ 'b': { count: 1, userFacedAt: null } }).map(([rxn, { count, userFacedAt: userFacedAtTs }]) =>
        <form
          key={gram.id}
          className="inline-block"
          action={
            userFacedAtTs ? Actions.unface : Actions.doface
          }
        >
          <input type="hidden" name="csrf_token" value={csrfToken} />
          <input name="epigramId" type="hidden" value={gram.id} />
          <input name="rxn" type="hidden" value={rxn} />
          <button
            type="submit"
            className="submitFacesButton p-1 b-1 rounded-2 b-outline hover:b-black dark:hover:b-white data-[faced]:b-orange"
            {...(userFacedAtTs ? { 'data-faced': !!userFacedAtTs } : {})}
          >
            <span className="">{rxn}</span>
            <span className="italic">{count}</span>
          </button>
        </form>
      )}
    </div>
    <div dangerouslySetInnerHTML={{ __html: gram.content }}></div>
    <Link href="#">reply</Link>
    <ul className="ml-4">
      {
        gram.replies?.map(
          gram => <li key={gram.id}> <Comment gram={gram} csrfToken={getCsrfToken()} /> </li>
        )
      }
    </ul>
  </>
}

