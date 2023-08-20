"use client"

import { RadFControl, RadFField, RadFLabel, RadFMessage, RadFRoot, RadFSubmit } from "@/app/_components/radix"
import { T } from "@/client";
import { submitPost } from './actions';
import { useState } from 'react';

type DePromisify<T> = T extends Promise<infer Inner> ? Inner : T;
type ActionErr<A extends (...args: any) => any> = DePromisify<ReturnType<A>> | undefined;

export function SumbitPostForm({
  redirectTo,
  csrfToken,
}: {
  redirectTo: string,
  csrfToken: string,
}) {
  const [serverErr, setServerErr] = useState<ActionErr<typeof submitPost>>(undefined);

  return (
    <>
      <RadFRoot asChild>
        <form
          action={async (formData) => setServerErr(await submitPost(formData))}
          className="flex flex-col gap-2">
          <input type="hidden" name="csrf_token" value={csrfToken} />
          {
            serverErr?.formError &&
            <div className="">
              {serverErr?.formError}
            </div>
          }
          <input type="hidden" name="redirectTo" value={redirectTo} />
          <RadFField
            name="title"
          >
            <div
              className="flex"
            >
              <RadFLabel className="w-20%">
                Title
              </RadFLabel>
              <RadFControl
                type="text"
                required
                maxLength={T.MAX_LENGTH_TITLE}
                className="w-full"
              />
            </div>
            <div>
              <RadFMessage match="valueMissing">
                Title is missing.
              </RadFMessage>
              <RadFMessage match="tooLong">
                Title is too short. Can&apos;t be longer than {T.MAX_LENGTH_TITLE}.
              </RadFMessage>
            </div>
          </RadFField>
          <RadFField
            name="url"
          >
            <div
              className="flex"
            >
              <RadFLabel className="w-20%">
                Url
              </RadFLabel>
              <RadFControl
                type="url"
                className="w-full"
              />
            </div>
            <div>
              <RadFMessage match="rangeOverflow">
                Provided url is not valid
              </RadFMessage>
            </div>
          </RadFField>
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
                <textarea rows={8} cols={80} />
              </RadFControl>
            </div>
          </RadFField>
          <RadFSubmit>
            Submit
          </RadFSubmit>
        </form>
      </RadFRoot >
    </>
  );
}
