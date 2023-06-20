import { atom, } from "jotai";
import { queryClientAtom } from "jotai-tanstack-query";

import { ApiClient } from "./";

// NOTE: hydrate these from loaders
export const aggyBaseUrlAtom = atom("NOT_SET");

export const apiClientAtom = atom(
  get => new ApiClient(
    get(aggyBaseUrlAtom),
    get(queryClientAtom),
  )
);
