import { json, type ActionFunction } from '@remix-run/node';
import { isTheme } from '~/utils/theme';
import { getThemeSession, } from '~/utils/theme.server';

export const action: ActionFunction = async ({ request }) => {
  const themeSession = await getThemeSession(request);
  const requestText = await request.text();
  const form = new URLSearchParams(requestText);
  const theme = form.get('theme');

  if (!isTheme(theme)) {
    return json({}, { status: 400 });
  }

  themeSession.setTheme(theme);
  return json({}, { headers: { 'Set-Cookie': await themeSession.commit() } });
}
