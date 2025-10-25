/// Allows you to bypass the authentication restrictions for navigating between pages.

import { createContext } from "react";
import { BYPASS_PROTECTION } from "./config";
import type { GlobalState } from "../components/GlobalProvider";

/// This does not bypass actual authentication.
export function bypassProtection(): boolean {
	return import.meta.env.DEV && BYPASS_PROTECTION;
}

export const GlobalContext = createContext<GlobalState | null>(null);

export type ApiResult<T> = {
	result: T | null;
	status: number;
};
