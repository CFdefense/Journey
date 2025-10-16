import { useNavigate, Link } from "react-router-dom";
import { apiLogout } from "../api/account";

export default function Account() {
  const navigate = useNavigate();

  const onLogout = async () => {
    console.log("Logging out");
    try {
      // Wait for the backend to actually delete the cookie
      await apiLogout();

      // Optionally force a full reload to clear client state and re-check cookie
      window.location.href = "/"; // safer than navigate() for auth resets
    } catch (e) {
      console.error("Logout error:", e);
      // Still go home, but log error for debugging
      window.location.href = "/";
    }
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
