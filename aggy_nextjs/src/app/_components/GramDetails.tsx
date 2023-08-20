"use client"

import { useState } from 'react';
import Link from 'next/link';

import { RadFControl, RadFField, RadFLabel, RadFRoot, RadFSubmit } from "@/app/_components/radix"
import * as T from "@/client/types";
import * as Actions from "@/app/api/actions";
import { ActionErr } from '@/utils';


export function GramDetails(
  { gram, csrfToken }: { gram: T.Gram; csrfToken: string; }
) {
  const [serverErr, setServerErr] = useState<ActionErr<typeof Actions.reply>>(undefined);

  return <div className="flex flex-col gap-2">
    <div dangerouslySetInnerHTML={{ __html: gram.content }}></div>
    <div>by {gram.authorAlias ?? gram.authorPubkey}</div>
    <RadFRoot asChild>
      <form
        action={async (formData) => setServerErr(await Actions.reply(formData))}
        className="flex flex-col gap-2"
      >
        <input type="hidden" name="csrf_token" value={csrfToken} />
        <input name="parentId" value={gram.id} type="hidden" />
        {
          serverErr?.formError &&
          <div className="">
            {serverErr?.formError}
          </div>
        }
        <RadFField
          name="body"
        >
          <div
            className="flex"
          >
            <RadFLabel className="w-20%">
              Body
            </RadFLabel>
            <RadFControl
              type="text"
              className="w-full"
              asChild
            >
              <textarea cols={80} rows={5} />
            </RadFControl>
          </div>
        </RadFField>
        <RadFSubmit>
          add comment
        </RadFSubmit>
      </form>
    </RadFRoot >
    <br />
    <ul>{gram.replies?.map(
      eg => <li key={eg.id}>
        <Gram gram={eg} csrfToken={csrfToken} />
      </li>
    )}</ul>
  </div>;
}

export function Gram({ gram, csrfToken }: { gram: T.Gram, csrfToken: string, }) {
  return <>
    <div>
      <Link href={`/user/${gram.authorPubkey}`}>{gram.authorAlias ?? gram.authorPubkey}</Link>
      |
      {new Date(gram.createdAt).toLocaleString()}

      {/* Object.entries({ 'b': { count: 1, userFacedAt: null } }).map(([rxn, { count, userFacedAt: userFacedAtTs }]) =>
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
      ) */}
    </div>
    <div dangerouslySetInnerHTML={{ __html: gram.content }}></div>
    <Link href={`/g/${gram.id}`}>reply</Link>
    <ul className="ml-4">
      {
        gram.replies?.map(
          gram => <li key={gram.id}> <Gram gram={gram} csrfToken={csrfToken} /> </li>
        )
      }
    </ul>
  </>
}

