import * as Separator from '@radix-ui/react-separator';
import { Link } from '@remix-run/react';
import React from 'react';

import * as T from "~/api/types";

export { Image, type ImageProps } from "@unpic/react";
export * as Toolbar from '@radix-ui/react-toolbar';
export * as Toggle from '@radix-ui/react-toggle';
export * as Tooltip from '@radix-ui/react-tooltip';
export * as RadixForm from '@radix-ui/react-form';

export const Divider = React.forwardRef<HTMLDivElement, Separator.SeparatorProps>(
  function Divider(props, ref) {
    return (
      <Separator.Root
        className="divider"
        ref={ref}
        {...props}
      />
    );
  }
)

export function PostStatusLines({ post }: { post: T.AggyPost }) {
  return <div className="postStatusLines">
    <div className="postStatusDetailsLine flex gap-1">
      <span>
        by <Link to={`/user/${post.epigram.author.pkey}`}>{post.epigram.author.alias}</Link>
      </span>
      |
      <a href={`/p/${post.id}`}>{post.commentCount} comments</a>
    </div>
    <div className="postStatusFacesLine flex gap-1">
      {Object.entries(post.epigram.topFaces).map(([rxn, { count, userFacedAt: userFacedAtTs }]) =>
        <form
          key={post.id}
          className="inline-block"
          action={
            userFacedAtTs ?
              `/api/unface`
              : `/api/doface`
          }
        >
          <input name="epigramId" type="hidden" value={post.epigram.id} />
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
