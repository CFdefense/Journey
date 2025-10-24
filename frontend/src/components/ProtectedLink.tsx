import { Link, type To } from "react-router-dom";
import "../styles/Index.css";
import { useContext, useEffect, useState, type Context } from "react";
import { bypassProtection, GlobalContext } from "../helpers/global";
import { apiValidate } from "../api/account";
import { Loading } from "../components/Loading";
import type { GlobalState } from "./GlobalProvider";

interface ProtectedLinkParams {
  authTo: To;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  authChildren: any;
  unauthTo: To;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  unauthChildren: any;
}

/// A link that changes depending on authentication status.
// If authenticated, it will href authTo and display authChildren.
// If not authenticated, it will href unauthTo and display unauthChildren.
export function ProtectedLink({
  authTo,
  authChildren,
  unauthTo,
  unauthChildren
}: ProtectedLinkParams) {
  const [loading, setLoading] = useState(true);
  const { authorized, setAuthorized } = useContext<GlobalState>(
    GlobalContext as Context<GlobalState>
  );

  useEffect(() => {
    if (!bypassProtection()) {
      if (authorized !== null) {
        setLoading(false);
        return;
      }
      apiValidate()
        .then(({ result, status }) => {
          setAuthorized(result && status === 200);
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
    if (authorized === true) {
      return (
        <Link to={authTo} replace>
          {authChildren}
        </Link>
      );
    } else {
      return (
        <Link to={unauthTo} replace>
          {unauthChildren}
        </Link>
      );
    }
  } else {
    return (
      <Link to={authTo} replace>
        {authChildren}
      </Link>
    );
  }
}
