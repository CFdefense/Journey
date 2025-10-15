import { Link } from "react-router-dom";

export default function Home() {
  return (
    <div>
      {/* Navigation */}
      <nav>
        <Link to="/">Index</Link>| <Link to="/view">View</Link>|{" "}
        <Link to="/account">Account</Link>
      </nav>
      <h1>Home</h1>
    </div>
  );
}
