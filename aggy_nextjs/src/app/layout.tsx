import type { Metadata } from 'next'
import Link from "next/link"

import './globals.css';
import '@unocss/reset/tailwind.css';
// import { Inter } from 'next/font/google'

// const inter = Inter({ subsets: ['latin'] })

export const metadata: Metadata = {
  title: {
    template: '%s | Aggy',
    default: 'Aggy'
  },
  description: 'An experiment.',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className={""/* inter.className */}>
        <header className="flex gap-2 px-2rem py-2rem">
          <Link href="/">Aggy</Link>
          <nav className="mx-a flex gap-2">
            <a href="new">new</a>
            <a href="comments">comments</a>
            <a href="submit">submit</a>
          </nav>
        </header>
        <main>
          {children}
        </main >
      </body>
    </html>
  )
}
