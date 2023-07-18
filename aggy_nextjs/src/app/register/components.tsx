"use client"

import { NextResponse } from 'next/server'
import * as zod from "zod";
import { fromZodError } from "zod-validation-error";

import { RadFControl, RadFField, RadFLabel, RadFMessage, RadFRoot, RadFSubmit } from "@/app/_components/radix"
import {
  CreateUserInput,
  MAX_LENGTH_PASSWORD, MAX_LENGTH_USERNAME, MIN_LENGTH_PASSWORD, MIN_LENGTH_USERNAME
} from "@/client";
import { dbg } from "@/utils";
import { apiClient } from '@/client/index.server';
import { register } from './actions';
import { useState } from 'react';

type DePromisify<T> = T extends Promise<infer Inner> ? Inner : T;
type ActionErr<A extends (...args: any) => any> = DePromisify<ReturnType<A>> | undefined;

export function RegisterForm({
  redirectTo,
}: {
  redirectTo: string,
}) {
  const [serverErr, setServerErr] = useState<ActionErr<typeof register>>(undefined);

  return (
    <>
      <RadFRoot asChild>
        <form
          action={async (formData) => setServerErr(await register(formData))}
          className="flex flex-col gap-2">
          <h3>Register</h3>
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
                minLength={MIN_LENGTH_USERNAME}
                maxLength={MAX_LENGTH_USERNAME}
                className="w-full"
              />
            </div>
            <div>
              <RadFMessage match="valueMissing">
                Username is missing.
              </RadFMessage>
              <RadFMessage match="tooShort">
                Username is too short. Must be at least {MIN_LENGTH_USERNAME} chars long..
              </RadFMessage>
              <RadFMessage match="tooLong">
                Username is too short. Must be at least {MAX_LENGTH_USERNAME} chars long..
              </RadFMessage>
            </div>
          </RadFField>
          <RadFField
            name="email"
          >
            <div
              className="flex"
            >
              <RadFLabel className="w-20%">
                Email
              </RadFLabel>
              <RadFControl type="email" required className="w-full" />
            </div>
            <div>
              <RadFMessage match="valueMissing">
                Email is missing.
              </RadFMessage>
              <RadFMessage match="typeMismatch">
                Provided email is invalid.
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
                minLength={MIN_LENGTH_PASSWORD}
                maxLength={MAX_LENGTH_PASSWORD}
                className="w-full"
              />
            </div>
            <div>
              <RadFMessage match="valueMissing">
                Password is missing.
              </RadFMessage>
              <RadFMessage match="tooShort">
                Password is too short. Must be at least {MIN_LENGTH_PASSWORD} chars long..
              </RadFMessage>
              <RadFMessage match="tooLong">
                Password is too short. Must be at least {MAX_LENGTH_PASSWORD} chars long..
              </RadFMessage>
            </div>
          </RadFField>
          <RadFSubmit>
            Register
          </RadFSubmit>
        </form>
      </RadFRoot >
    </>
  );
}
