import { cookies } from 'next/headers'
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

  private async readSession() {
    const cookieStore = cookies();
    const jwt = cookieStore.get(SESSION_COOKIE)?.value;
    if (!jwt) {
      return;
    }
    const payload = jwtPayloadValidator.parse(
      (await jose.jwtVerify(jwt, SESSION_SECRET)).payload
    )
    this.sessionCache = {
      id: payload.sid
    }
  }

  async id() {
    let session = this.sessionCache;
    if (!session) {
      await this.readSession();
      session = this.sessionCache!;
    }
    return session.id;
  }

  async load() {
    let session = this.sessionCache;
    if (!session) {
      await this.readSession();
      session = this.sessionCache!;
    }
    if (!("expiresAt" in session)) {
      this.sessionCache = await this.aggy.getSession(session.id)!;
      session = this.sessionCache!;
    }
    return session as T.Session;
  }

  async add(request: NextRequest, response: NextResponse) {
    if (request.cookies.has(SESSION_COOKIE)) {
      return
    }
    const input: T.CreateSessionBody = {
      userAgent: userAgent({ headers: request.headers }).ua,
      ipAddr: request.headers.get('x-forwarded-for')!,
    };
    const resp = await this.aggy.createSession(input);
    const jwt = await new jose.SignJWT({
      sid: resp.id,
    })
      .setProtectedHeader({ alg: 'HS256' })
      .setExpirationTime(new Date(resp.expiresAt).valueOf())
      .setIssuer('aggy_nextjs')
      .sign(SESSION_SECRET);
    response.cookies.set(
      {
        name: SESSION_COOKIE,
        value: jwt,
        secure: process.env.NODE_ENV === 'production',
        httpOnly: true,
        expires: new Date(resp.expiresAt),
        sameSite: "strict",
        path: '/'
      }
    );
  }
}

export async function addSessionMiddleware(request: NextRequest, response: NextResponse) {
  const { session } = apiClient()
  await session.add(request, response);
}
