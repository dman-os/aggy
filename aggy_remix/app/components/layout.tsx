import { Link } from "@remix-run/react";

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className={""/* inter.className */}>
        <header className="flex justify-between gap-2 px-2rem py-2rem">
          <Link to="/">aggy</Link>
          <nav className="flex gap-2">
            <Link to="/new">new</Link>
            <Link to="/comments">comments</Link>
            <Link to="/submit">submit</Link>
          </nav>
          <div>
            <Link to="/login">login</Link>
            /
            <Link to="/register">register</Link>
          </div>
        </header>
        <main>
          {children}
        </main >
      </body>
    </html>
  )
}
