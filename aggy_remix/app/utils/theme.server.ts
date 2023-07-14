

import { createCookieSessionStorage } from '@remix-run/node';

import { type Theme, isTheme } from "./theme"
import { SESSION_SECRET } from "./index.server";

const themeStorage = createCookieSessionStorage({
  cookie: {
    name: 'SA_theme',
    secure: true,
    secrets: [SESSION_SECRET],
    sameSite: 'lax',
    path: '/',
    httpOnly: true,
  },
});

export async function getThemeSession(request: Request) {
  const session = await themeStorage.getSession(request.headers.get('Cookie'));
  return {
    getTheme: () => {
      const themeValue = session.get('theme');
      return isTheme(themeValue) ? themeValue : null;
    },
    setTheme: (theme: Theme) => session.set('theme', theme),
    commit: () => themeStorage.commitSession(session),
  };
}


