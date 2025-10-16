import { Link, type To } from "react-router-dom";
import "../styles/Index.css";
import { useEffect, useState } from "react";
import { bypassProtection } from "../helpers/global";
import { apiValidate } from "../api/account";
import { Loading } from "../components/Loading";

interface ProtectedLinkParams {
  authTo: To,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  authChildren: any,
  unauthTo: To,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  unauthChildren: any
}

/// A link that changes depending on authentication status.
// If authenticated, it will href authTo and display authChildren.
// If not authenticated, it will href unauthTo and display unauthChildren.
export function ProtectedLink({ authTo, authChildren, unauthTo, unauthChildren }: ProtectedLinkParams) {
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
    if (authorized) {
      return <Link to={authTo}>{authChildren}</Link>;
    } else {
      return <Link to={unauthTo}>{unauthChildren}</Link>;
    }
  } else {
    return <Link to={authTo}>{authChildren}</Link>;
  }
}