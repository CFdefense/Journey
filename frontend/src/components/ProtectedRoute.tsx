import { useContext, useEffect, useRef, type Context } from "react";
import { apiValidate } from "../api/account";
import { Navigate } from "react-router-dom";
import { Loading } from "./Loading";
import { bypassProtection, GlobalContext } from "../helpers/global";
import type { GlobalState } from "./GlobalProvider";

/// Only allows access to the child elements if authenticated.
/// Displays a loading screen while verifying.
/// Redirects to login if not authenticated.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function ProtectedRoute({ children }: any) {
  const { authorized, setAuthorized } = useContext<GlobalState>(
    GlobalContext as Context<GlobalState>
  );
  const isCheckingRef = useRef(false);

  useEffect(() => {
    if (!bypassProtection()) {
      if (authorized !== null) {
        isCheckingRef.current = false;
        return;
      }
      // Only set checking flag when we actually start the API call
      if (!isCheckingRef.current) {
        isCheckingRef.current = true;
        apiValidate()
          .then(({ status }) => {
            setAuthorized(status === 200);
          })
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          .catch((err: any) => {
            setAuthorized(false);
            console.error(err);
          })
          .finally(() => {
            isCheckingRef.current = false;
          });
      }
    }
  }, [authorized, setAuthorized]);

  if (!bypassProtection()) {
    // Only show loading if we're actively checking (authorized is null AND we've started checking)
    if (authorized === null && isCheckingRef.current) return <Loading />;
    if (authorized === false) return <Navigate to="/login" replace />;
  }

  return children;
}

/// Only allows access to the child elements if not authenticated.
/// Displays a loading screen while verifying.
/// Redirects to home if authenticated.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function InverseProtectedRoute({ children }: any) {
  const { authorized, setAuthorized } = useContext<GlobalState>(
    GlobalContext as Context<GlobalState>
  );
  const isCheckingRef = useRef(false);

  useEffect(() => {
    if (!bypassProtection()) {
      if (authorized !== null) {
        isCheckingRef.current = false;
        return;
      }
      // Only set checking flag when we actually start the API call
      if (!isCheckingRef.current) {
        isCheckingRef.current = true;
        apiValidate()
          .then(({ status }) => {
            setAuthorized(status === 200);
          })
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          .catch((err: any) => {
            setAuthorized(false);
            console.error(err);
          })
          .finally(() => {
            isCheckingRef.current = false;
          });
      }
    }
  }, [authorized, setAuthorized]);

  if (!bypassProtection()) {
    // Only show loading if we're actively checking (authorized is null AND we've started checking)
    if (authorized === null && isCheckingRef.current) return <Loading />;
    if (authorized === true) return <Navigate to="/home" replace />;
  }

  return children;
}
