import { useContext, useEffect, useState, type Context } from "react";
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
  const [loading, setLoading] = useState(true);
  const { authorized, setAuthorized } = useContext<GlobalState>(GlobalContext as Context<GlobalState>);

  useEffect(() => {
    if (!bypassProtection()) {
      if (authorized !== null) {
        setLoading(false);
        return;
      }
      apiValidate()
        .then((valid: boolean) => {
          setAuthorized(valid);
        })
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        .catch((err: any) => {
          setAuthorized(false);
          console.error(err);
        })
        .finally(() => {
          setLoading(false);
        });
    }
  }, [authorized, setAuthorized]);

  if (!bypassProtection()) {
    console.log("loading: ", loading);
    console.log("authorized: ", authorized);
    if (loading) return <Loading />;
    if (!authorized) return <Navigate to="/login" replace />;
  }

  return children;
}

/// Only allows access to the child elements if not authenticated.
/// Displays a loading screen while verifying.
/// Redirects to home if authenticated.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function InverseProtectedRoute({ children }: any) {
  const [loading, setLoading] = useState(true);
  const { authorized, setAuthorized } = useContext<GlobalState>(GlobalContext as Context<GlobalState>);

  useEffect(() => {
    if (!bypassProtection()) {
      if (authorized !== null) {
        setLoading(false);
        return;
      }
      apiValidate()
        .then((valid: boolean) => {
          setAuthorized(valid);
        })
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        .catch((err: any) => {
          setAuthorized(false);
          console.error(err);
        })
        .finally(() => {
          setLoading(false);
        });
    }
  }, [authorized, setAuthorized]);

  if (!bypassProtection()) {
    if (loading) return <Loading />;
    if (authorized) return <Navigate to="/home" replace />;
  }

  return children;
}
