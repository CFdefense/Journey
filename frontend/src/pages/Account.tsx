import { Link } from "react-router-dom";
import { apiLogout } from "../api/account";
import { useContext, type Context } from "react";
import { GlobalContext } from "../helpers/global";
import type { GlobalState } from "../components/GlobalProvider";
import { ACTIVE_CHAT_SESSION } from "./Home";

export default function Account() {
  const { setAuthorized } = useContext<GlobalState>(
    GlobalContext as Context<GlobalState>
  );

  const onLogout = async () => {
    console.log("Logging out");
    const { status } = await apiLogout();
    if (status !== 200) {
      console.error("Logout failed with status", status);
    }
    setAuthorized(false);
    sessionStorage.removeItem(ACTIVE_CHAT_SESSION);
  };

  return (
    <div>
      {/* Navigation */}
      <nav>
        <Link to="/">Index</Link> | <Link to="/home">Home</Link> |{" "}
        <Link to="/view">View</Link>
      </nav>

      <h1>Account Page</h1>
      <button onClick={onLogout}>Logout</button>
    </div>
  );
}
