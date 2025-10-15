import { useEffect, useState } from "react";
import { apiValidate } from "../api/account";
import { Navigate } from "react-router-dom";
import { Loading } from "./Loading";
import { bypassProtection } from "../helpers/global";

/// Only allows access to the child elements if authenticated.
/// Displays a loading screen while verifying.
/// Redirects to login if not authenticated.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function ProtectedRoute({ children }: any) {
  const [loading, setLoading] = useState(true);
  const [authorized, setAuthorized] = useState(false);

  useEffect(() => {
    if (!bypassProtection()) {
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
  }, []);

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
  const [authorized, setAuthorized] = useState(false);

  useEffect(() => {
    if (!bypassProtection()) {
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
  }, []);

  if (!bypassProtection()) {
    if (loading) return <Loading />;
    if (authorized) return <Navigate to="/home" replace />;
  }

  return children;
}