import { cookies } from 'next/headers'
import * as jose from "jose";
import * as zod from "zod";

import { AggyClient, T } from "@/client";
import { assertNotNull, dbg } from '.';
import { apiClient } from '@/client/index.server';
import { NextRequest, NextResponse, userAgent } from 'next/server';

const SESSION_SECRET = new TextEncoder().encode(assertNotNull(process.env.SESSION_SECRET));
const SESSION_COOKIE = 'AGGY_session';

const jwtPayloadValidator = zod.object({
  sid: zod.string().uuid()
})

export class SessionStore {
  private sessionCache: { id: string; } | undefined;
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

  async add(request: NextRequest, response: NextResponse) {
    if (request.cookies.has(SESSION_COOKIE)) {
      return
    }
    const input: T.CreateSessionInput = {
      userAgent: userAgent({ headers: request.headers }).ua,
      ipAddr: request.headers.get('x-forwarded-for')!,
    };
    const resp = await this.aggy.createSession(input);
    const jwt = await new jose.SignJWT({
      sid: resp.id,
    })
      .setProtectedHeader({ alg: 'HS256' })
      .setExpirationTime(resp.expiresAt)
      .setIssuer('aggy_nextjs')
      .sign(SESSION_SECRET);
    response.cookies.set(
      {
        name: SESSION_COOKIE,
        value: jwt,
        secure: process.env.NODE_ENV === 'production',
        httpOnly: true,
        expires: new Date(resp.expiresAt * 1000),
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
