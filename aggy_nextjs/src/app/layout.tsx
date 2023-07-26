import type { Metadata } from 'next'
import Link from "next/link"

import './globals.css';
import '@unocss/reset/tailwind.css';
import { apiClient } from '@/client/index.server';

import { dbg } from '@/utils';
import { getCsrfToken } from '@/utils/index.server';

import { logout } from "./actions";

// import { Inter } from 'next/font/google'

// const inter = Inter({ subsets: ['latin'] })

export const metadata: Metadata = {
  title: {
    template: '%s | Aggy',
    default: 'Aggy'
  },
  description: 'An experiment.',
}

export default async function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  const { client, session } = apiClient();
  const { userId } = await session.load();

  const csrfToken = getCsrfToken();

  return (
    <html lang="en">
      <body className={""/* inter.className */}>
        <header className="flex justify-between gap-2 px-2rem py-2rem">
          <Link href="/">aggy</Link>
          <nav className="flex gap-2">
            <Link href="/new">new</Link>
            <Link href="/comments">comments</Link>
            <Link href="/submit">submit</Link>
          </nav>
          {
            userId ?
              <div>
                {userId}
                |
                <form
                  className="inline-block"
                  action={
                    logout
                  }
                >
                  <input type="hidden" name="csrf_token" value={csrfToken} />
                  <button
                    type="submit"
                  >
                    logout
                  </button>
                </form>
              </div>
              :
              <div>
                <Link href="/login">login</Link>
                /
                <Link href="/register">register</Link>
              </div>
          }
        </header>
        <main>
          {children}
        </main >
      </body>
    </html>
  )
}
