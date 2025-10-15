import { useNavigate, Link } from "react-router-dom";
import { apiLogout } from "../api/account";

export default function Account() {
  const navigate = useNavigate();

  const onLogout = async () => {
    console.log("Logging out");
    try {
      apiLogout();
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
    } catch (e: any) {
      console.error(e);
    }
    navigate("/");
  };

  return (
    <div>
      {/* Navigation */}
      <nav>
        <Link to="/">Index</Link>| <Link to="/home">Home</Link>|{" "}
        <Link to="/view">View</Link>
      </nav>
      <h1>Account Page</h1>
      <button onClick={onLogout}>Logout</button>
    </div>
  );
}
