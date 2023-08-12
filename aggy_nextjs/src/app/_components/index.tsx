export * from './radix';

import Link from 'next/link';

import * as T from "@/client/types";
import * as Actions from "@/app/api/actions";

export function PostStatusLines({ post, csrfToken }: { post: T.Post, csrfToken: string, }) {
  return <div className="postStatusLines">
    <div className="postStatusDetailsLine flex gap-1">
      <span>
        by <Link href={`/user/${post.authorPubKey}`}>{post.authorUsername}</Link>
      </span>
      |
      <a href={`/p/${post.id}`}>X comments</a>
    </div>
    <div className="postStatusFacesLine flex gap-1">
      {Object.entries({ 'b': { count: 1, userFacedAt: null } }).map(([rxn, { count, userFacedAt: userFacedAtTs }]) =>
        <form
          key={post.id}
          className="inline-block"
          action={
            userFacedAtTs ? Actions.unface : Actions.doface
            // `/api/unface?epigram_id=${post.epigram.id}&rxn=${rxn}`
            // : `/api/doface?epigram_id=${post.epigram.id}&rxn=${rxn}`
          }
        >
          <input type="hidden" name="csrf_token" value={csrfToken} />
          <input name="epigramId" type="hidden" value={post.epigramId} />
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
  </div>
}
