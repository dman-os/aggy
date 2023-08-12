import { cookies, headers } from 'next/headers'
import * as jose from "jose";
import * as zod from "zod";

import { AggyClient, T } from "@/client";
import { assertNotNull, } from '.';
import { apiClient } from '@/client/index.server';
import { NextRequest, NextResponse, userAgent } from 'next/server';

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

  private async newSession() {
    const hdrs = headers();
    const input: T.CreateSessionBody = {
      userAgent: userAgent({ headers: hdrs }).ua,
      ipAddr: hdrs.get('x-forwarded-for')!,
    };
    return await this.aggy.createSession(input);
  }

  async id() {
    let session = this.sessionCache;
    if (!session) {
      session = await readCookieSession();
      // if nothing in the cookie, create a new one
      if (!session) {
        const fullSession = await this.newSession();
        await addCookieSession(fullSession);
        session = fullSession;
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
        session = await this.aggy.getSession(session.id)!;
      }
    }
    this.sessionCache = session;
    return session as T.Session;
  }
}

async function addCookieSession(session: T.Session) {
  const jwt = await new jose.SignJWT({
    sid: session.id,
  })
    .setProtectedHeader({ alg: 'HS256' })
    .setExpirationTime(new Date(session.expiresAt).valueOf())
    .setIssuer('aggy_nextjs')
    .sign(SESSION_SECRET);
  cookies().set(
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
    id: payload.sid
  }
}
