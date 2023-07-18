import Link from "next/link";

import {
  RadFControl, RadFField, RadFLabel, RadFMessage, RadFRoot, RadFSubmit
} from "@/app/_components/radix"
import { login } from "@/app/api/actions";
import { MAX_LENGTH_PASSWORD, MAX_LENGTH_USERNAME, MIN_LENGTH_PASSWORD, MIN_LENGTH_USERNAME } from "@/client";

export default function LoginPage() {
  return (
    <>
      <RadFRoot asChild>
        <form action={login} className="flex flex-col gap-2">
          <h3>Login</h3>
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
            name="password"
          >
            <div
              className="flex"
            >
              <RadFLabel className="w-20%">
                Password
              </RadFLabel>
              <RadFControl
                type="text"
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
          <RadFSubmit>Login</RadFSubmit>
        </form>
      </RadFRoot >
      {/* <h3>Create Account</h3> */}
      <Link href="/register">register</Link>
    </>
  );
}
