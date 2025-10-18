import { Link } from "react-router-dom";
import { ProtectedLink } from "../components/ProtectedLink";

export default function IndexPage() {
  return (
    <div>
      {/* Navigation */}
      <nav>
        <Link to="/">Index</Link>{" "}
        | <ProtectedLink
            authTo="/home"
            authChildren={<div>Home</div>}
            unauthTo="/login"
            unauthChildren={<div>Login</div>}
          />{" "}
        | <ProtectedLink
            authTo="/Account"
            authChildren={<div>Account</div>}
            unauthTo="/signup"
            unauthChildren={<div>Sign Up</div>}
          />
      </nav>
      <h1>Welcome to Travel Planner</h1>
    </div>
  );
}