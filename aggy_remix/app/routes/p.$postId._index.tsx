
import { PostStatusLines } from "~/components";
import * as T from "~/api/types";
import { useLoaderData } from "@remix-run/react";
import { newApiClient } from "~/api/index.server";
import type { LoaderArgs } from "@remix-run/node";

export const loader = async ({ request, params }: LoaderArgs) => {
  const { response, client } = newApiClient({ request });

  const postId = params["postID"];

  if (!postId) {
    return response.redirect("/404");
  }
  const topPosts = await client.getTopPosts();
  const post = topPosts.find((post) => post.id === params["post-id"]);
  if (!postId) {
    return response.redirect("/404");
  }
  return response.ok({
    post
  });
}

export default function PostDetails() {
  const { post } = useLoaderData<typeof loader>();
  return <div className="flex flex-col gap-2">
    <div dangerouslySetInnerHTML={{ __html: post.epigram.content }}></div>
    <PostStatusLines post={post} />
    <form className="addCommentForm flex flex-col gap-1">
      <input name="parentId" value={post.epigram.id} type="hidden" />
      <textarea name="content" cols={80} rows={5} />
      <button type="submit" className="self-start p-1 b-1 rounded-2 b-outline hover:b-black dark:hover:b-white">
        add comment
      </button>
    </form>
    <br />
    <ul>{post.epigram.children.map(
      eg => <li key={eg.id}>
        <Comment epigram={eg} />
      </li>
    )}</ul>
  </div>
}

function Comment({ epigram }: { epigram: T.Epigram }) {
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
    <Link to="#">reply</Link>
    <ul className="ml-4">
      {
        epigram.children.map(
          eg => <li key={eg.id}> <Comment epigram={eg} /> </li>
        )
      }
    </ul>
  </>
}


