import { Link } from "react-router-dom";
import { apiLogout } from "../api/account";
import { useContext, type Context } from "react";
import { GlobalContext } from "../helpers/global";
import type { GlobalState } from "../components/GlobalProvider";

export default function Account() {
  const { setAuthorized } = useContext<GlobalState>(
    GlobalContext as Context<GlobalState>
  );

  const onLogout = async () => {
    console.log("Logging out");
    const { status, result } = await apiLogout();
    if (!result || status !== 200) {
      console.error("Logout failed with status", status);
    }
    setAuthorized(false);
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
