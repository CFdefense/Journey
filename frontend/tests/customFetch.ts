import makeFetchCookie from "fetch-cookie";

/// Fetch only stores cookies when running in the browser.
/// When we run tests, it's running with Node, not the browser.
/// Node's implementation of fetch doesn't store cookies,
/// so we have to make a wrapper for it.
export const customFetch = makeFetchCookie(fetch);