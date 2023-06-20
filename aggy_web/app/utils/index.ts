export { useQueryFam } from "./atoms";

export function without<O extends Object, K extends keyof O>(
  object: O,
  ...keys: Array<K>
) {
  const set = new Set(keys);
  return Object.fromEntries(
    Object.entries(object)
      .filter(([key, _]) => !set.has(key as K))
  );
}
