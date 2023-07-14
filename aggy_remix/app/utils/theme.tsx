import { useFetcher } from "@remix-run/react";
import { atom, useAtom } from "jotai";
import React from "react";

export enum Theme {
  DARK = 'dark',
  LIGHT = 'light',
}
const themes: Array<Theme> = Object.values(Theme);

export function isTheme(value: unknown): value is Theme {
  return typeof value === 'string' && themes.includes(value as Theme);
}

const prefersDarkMQ = "(prefers-color-scheme: dark)";
const clientThemeCode = `
;(() => {
  const theme = window.matchMedia(${JSON.stringify(prefersDarkMQ)}).matches
    ? 'dark'
    : 'light';
  const cl = document.documentElement.classList;
  const themeAlreadyApplied = cl.contains('light') || cl.contains('dark');
  if (themeAlreadyApplied) {
    // this script shouldn't exist if the theme is already applied!
    console.warn(
      "Hi there, could you let me know you're seeing this message? Thanks!",
    );
  } else {
    cl.add(theme);
  }
  const meta = document.querySelector('meta[name=color-scheme]');
  if (meta) {
    if (theme === 'dark') {
      meta.content = 'dark light';
    } else if (theme === 'light') {
      meta.content = 'light dark';
    }
  } else {
    console.warn(
      "Hey, could you let me know you're seeing this message? Thanks!",
    );
  }
})();
`;

export const rawThemeAtom = atom<Theme | null>(null);
export const themeAtom = atom(
  (get) => {
    let specifiedTheme = get(rawThemeAtom);
    if (specifiedTheme) {
      return specifiedTheme;
    }
    if (typeof document === 'undefined') {
      return null;
    }
    return window.matchMedia(prefersDarkMQ).matches ? Theme.DARK : Theme.LIGHT;
  },
  (_get, set, newVal: Theme) => {
    set(rawThemeAtom, newVal);
  }
);

export function ThemeHead({ hasCookieTheme }: { hasCookieTheme: boolean }) {
  const [theme, setTheme] = useAtom(themeAtom);

  const persistTheme = useFetcher();

  // TODO: remove this when persistTheme is memoized properly
  const persistThemeRef = React.useRef(persistTheme);
  React.useEffect(() => {
    persistThemeRef.current = persistTheme;
  }, [persistTheme]);

  const mountRun = React.useRef(false);

  React.useEffect(() => {
    if (!mountRun.current) {
      mountRun.current = true;
      return;
    }
    if (!theme) {
      return;
    }

    persistThemeRef.current.submit(
      { theme },
      { action: "action/set-theme", method: "post" }
    );
  }, [theme]);

  React.useEffect(() => {
    const mediaQuery = window.matchMedia(prefersDarkMQ);
    const handleChange = () => {
      setTheme(mediaQuery.matches ? Theme.DARK : Theme.LIGHT);
    };
    mediaQuery.addEventListener("change", handleChange);
    return () => mediaQuery.removeEventListener("change", handleChange);
  }, [setTheme]);

  return <>
    <meta name="color-scheme" content={theme === Theme.LIGHT ? 'light dark' : 'dark light'} />
    {
      !hasCookieTheme &&
      <script dangerouslySetInnerHTML={{ __html: clientThemeCode }} />
    }
  </>;
}



