"use client"

import { useState } from 'react';

import { RadFControl, RadFField, RadFLabel, RadFMessage, RadFRoot, RadFSubmit } from "@/app/_components/radix"
import { T } from "@/client";
import { login } from './actions';
import { ActionErr } from '@/utils';

export function LoginForm({
  redirectTo,
  csrfToken,
}: {
  redirectTo: string,
  csrfToken: string,
}) {
  const [serverErr, setServerErr] = useState<ActionErr<typeof login>>(undefined);

  return (
    <>
      <RadFRoot asChild>
        <form
          action={async (formData) => setServerErr(await login(formData))}
          className="flex flex-col gap-2"
        >
          <input type="hidden" name="csrf_token" value={csrfToken} />
          {
            serverErr?.formError &&
            <div className="">
              {serverErr?.formError}
            </div>
          }
          <input type="hidden" name="redirectTo" value={redirectTo} />
          <RadFField
            name="username"
          >
            <div
              className="flex"
            >
              <RadFLabel className="w-20%">
                Username
              </RadFLabel>
              <RadFControl
                type="text"
                required
                minLength={T.MIN_LENGTH_USERNAME}
                maxLength={T.MAX_LENGTH_USERNAME}
                className="w-full"
              />
            </div>
            <div>
              <RadFMessage match="valueMissing">
                Username is missing.
              </RadFMessage>
              <RadFMessage match="tooShort">
                Username is too short. Must be at least {T.MIN_LENGTH_USERNAME} chars long..
              </RadFMessage>
              <RadFMessage match="tooLong">
                Username is too short. Must be at least {T.MAX_LENGTH_USERNAME} chars long..
              </RadFMessage>
            </div>
          </RadFField>
          <RadFField
            name="password"
          >
            <div
              className="flex"
            >
              <RadFLabel className="w-20%">
                Password
              </RadFLabel>
              <RadFControl
                type="password"
                required
                minLength={T.MIN_LENGTH_PASSWORD}
                maxLength={T.MAX_LENGTH_PASSWORD}
                className="w-full"
              />
            </div>
            <div>
              <RadFMessage match="valueMissing">
                Password is missing.
              </RadFMessage>
              <RadFMessage match="tooShort">
                Password is too short. Must be at least {T.MIN_LENGTH_PASSWORD} chars long..
              </RadFMessage>
              <RadFMessage match="tooLong">
                Password is too short. Must be at least {T.MAX_LENGTH_PASSWORD} chars long..
              </RadFMessage>
            </div>
          </RadFField>
          <RadFSubmit>
            Login
          </RadFSubmit>
        </form>
      </RadFRoot >
    </>
  );
}
