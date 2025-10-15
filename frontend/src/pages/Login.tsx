import { useState } from "react";
import { useNavigate, Link } from "react-router-dom";
import "../styles/LoginSignup.css";
import { apiLogin } from "../api/account";

export default function Login() {
  const [email, setEmail] = useState(""); // react hook to make sure that variable stays changed after React re-renders (gives components memory). https://react.dev/reference/react/useState
  const [password, setPassword] = useState("");
  const [error, setError] = useState(""); // for showing error messages
  const navigate = useNavigate();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault(); // stops the page from refreshing

    try {
      await apiLogin({ email, password });
      console.log("Login successful");
      setError("");
      navigate("/home");
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
    } catch (err: any) {
      console.error(err);
      setError(err.message || "Login failed.");
    }
  };

  return (
    <div className="login-container">
      {/* Navigation */}
      <nav>
        <Link to="/">Index</Link>
      </nav>
      <div className="login-box">
        <h1>Login</h1>
        <form onSubmit={handleSubmit}>
          <label>
            Email:
            <input
              type="text"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              required
            />
          </label>

          <label>
            Password:
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
            />
          </label>

          <button type="submit">Log In</button>
        </form>
        {error && <p style={{ color: "red" }}>{error}</p>}

        <div className="login-actions">
          <Link to="/signup">
            <button type="button">Create Account</button>
          </Link>
        </div>
      </div>
    </div>
  );
}
