/// Allows you to bypass the authentication restrictions for navigating between pages.

import { BYPASS_PROTECTION } from "./config";

/// This does not bypass actual authentication.
export function bypassProtection(): boolean {
  return import.meta.env.DEV && BYPASS_PROTECTION;
}