import { Link } from "react-router-dom";
import "../styles/Index.css";

export default function IndexPage() {
  return (
    <div>
      {/* Navigation */}
      <nav>
        <Link to="/">Index</Link>| <Link to="/login">Login</Link>|{" "}
        <Link to="/signup">Signup</Link>
      </nav>
      <h1>Welcome to Travel Planner</h1>
    </div>
  );
}
