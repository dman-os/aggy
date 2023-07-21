import Link from "next/link";
import { notFound } from "next/navigation";

import { PostStatusLines } from "@/app/_components";
import { topPosts } from "@/client";
import * as T from "@/client/types";
import * as Actions from "@/app/api/actions"
import { getCsrfToken } from "@/utils/index.server";

export default function PostDetails(
  { params }: { params: { 'post-id': string } }
) {
  const post = topPosts.find((post) => post.id === params["post-id"]);
  if (!post) {
    return notFound();
  }
  return <div className="flex flex-col gap-2">
    <div dangerouslySetInnerHTML={{ __html: post.epigram.content }}></div>
    <PostStatusLines post={post} csrfToken={getCsrfToken()} />
    <form action={Actions.comment} className="addCommentForm flex flex-col gap-1">
      <input type="hidden" name="csrf_token" value={getCsrfToken()} />
      <input name="parentId" value={post.epigram.id} type="hidden" />
      <textarea name="content" cols={80} rows={5} />
      <button type="submit" className="self-start p-1 b-1 rounded-2 b-outline hover:b-black dark:hover:b-white">
        add comment
      </button>
    </form>
    <br />
    <ul>{post.epigram.children.map(
      eg => <li key={eg.id}>
        <Comment epigram={eg} csrfToken={getCsrfToken()} />
      </li>
    )}</ul>
  </div>
}

function Comment({ epigram, csrfToken }: { epigram: T.Epigram, csrfToken: string, }) {
  return <>
    <div>
      <Link href={`/user/${epigram.author.pkey}`}>{epigram.author.alias}</Link>
      |
      {new Date(epigram.ts).toLocaleString()}

      {Object.entries(epigram.topFaces).map(([rxn, { count, userFacedAt: userFacedAtTs }]) =>
        <form
          key={epigram.id}
          className="inline-block"
          action={
            userFacedAtTs ? Actions.unface : Actions.doface
          }
        >
          <input type="hidden" name="csrf_token" value={csrfToken} />
          <input name="epigramId" type="hidden" value={epigram.id} />
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
    <div dangerouslySetInnerHTML={{ __html: epigram.content }}></div>
    <Link href="#">reply</Link>
    <ul className="ml-4">
      {
        epigram.children.map(
          eg => <li key={eg.id}> <Comment epigram={eg} csrfToken={getCsrfToken()} /> </li>
        )
      }
    </ul>
  </>
}

