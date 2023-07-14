
declare function dbg<T>(value: T, message?: string): T;
// type DbgFn<T = void> = (value: T, message?: string) => T;

interface Window {
  dbg: typeof dbg;
}

declare module NodeJS {
  interface Global {
    dbg: typeof dbg;
  }
}

