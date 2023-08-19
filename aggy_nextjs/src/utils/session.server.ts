import { cookies, } from 'next/headers'
import * as jose from "jose";
import * as zod from "zod";
import { userAgent, NextRequest, NextResponse } from 'next/server';

import { AggyClient, T, AggyApiError, } from "@/client";
import { apiClient } from "@/client/index.server";
import { assertNotNull, } from '.';
import { ResponseCookies } from 'next/dist/compiled/@edge-runtime/cookies';
import { redirect } from 'next/navigation';

const SESSION_SECRET = new TextEncoder().encode(assertNotNull(process.env.SESSION_SECRET));
const SESSION_COOKIE = 'AGGY_session';

const jwtPayloadValidator = zod.object({
  sid: zod.string()
})

/// FIXME: make this deduped across nextjs context
export class SessionStore {
  private sessionCache: { id: string; } | T.Session | undefined;
  constructor(
    public aggy: AggyClient
  ) { }

  async add(request: NextRequest, response: NextResponse) {
    if (request.cookies.has(SESSION_COOKIE)) {
      return;
    }
    const input: T.CreateSessionBody = {
      userAgent: userAgent({ headers: request.headers }).ua,
      ipAddr: request.headers.get('x-forwarded-for')!,
    };
    const session = await this.aggy.createSession(input);
    await addCookieSession(response.cookies, session);
  }

  async id() {
    let session = this.sessionCache;
    if (!session) {
      session = await readCookieSession();
      // if nothing in the cookie, create a new one
      if (!session) {
        redirect("")
        // redirect to a cookie adding call
        throw Error("todo");
        // const fullSession = await this.newSession();
        // await addCookieSession(fullSession);
        // session = fullSession;
      }
    }
    this.sessionCache = session;
    return session.id;
  }

  async load() {
    let session = this.sessionCache;
    if (!session) {
      await this.id(); // use `id` to read/init session
      session = this.sessionCache!;
      // if it only read the session id, fetch the full body
      if (!("expiresAt" in session)) {
        try {
          session = await this.aggy.getSession(session.id)!;
        } catch (err) {
          if (err instanceof AggyApiError && err.code == "notFound") {
            // redirect to a cookie adding call
            throw err;
            // const fullSession = await this.newSession();
            // await addCookieSession(fullSession);
            // session = fullSession;
          } else {
            throw err;
          }
        }
      }
    }
    this.sessionCache = session;
    return session as T.Session;
  }
}

async function addCookieSession(cookies: ResponseCookies, session: T.Session) {
  const jwt = await new jose.SignJWT({
    sid: session.id,
    // uname: session.
  })
    .setProtectedHeader({ alg: 'HS256' })
    .setExpirationTime(new Date(session.expiresAt).valueOf())
    .setIssuer('aggy_nextjs')
    .sign(SESSION_SECRET);
  cookies.set(
    {
      name: SESSION_COOKIE,
      value: jwt,
      secure: process.env.NODE_ENV === 'production',
      httpOnly: true,
      expires: new Date(session.expiresAt),
      sameSite: "strict",
      path: '/'
    }
  );
}

async function readCookieSession() {
  const cookieStore = cookies();
  const jwt = cookieStore.get(SESSION_COOKIE)?.value;
  if (!jwt) {
    return;
  }
  const payload = jwtPayloadValidator.parse(
    (await jose.jwtVerify(jwt, SESSION_SECRET)).payload
  )
  return {
    id: payload.sid as string,
    // username: payload.uname as string,
  }
}

export async function addSessionMiddleware(request: NextRequest, response: NextResponse) {
  const { session } = apiClient();
  await session.add(request, response);
}
