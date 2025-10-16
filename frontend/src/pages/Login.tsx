import { useState } from "react";
import { useNavigate } from "react-router-dom";
import "../styles/Login.css";
import { apiLogin } from "../api/account";

export default function Login() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const navigate = useNavigate();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await apiLogin({ email, password });
      setError("");
      navigate("/home");
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : "Login failed.";
      setError(message);
    }
  };

  return (
    <div className="auth-content">
      <div className="login-container">
        <div className="login-box">
          <h1>Log in to your account</h1>
          <form onSubmit={handleSubmit}>
            <label>
              Email
              <input
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
              />
            </label>
            <label>
              Password
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
              />
            </label>
            <button type="submit">Log In</button>
          </form>
          {error && <p>{error}</p>}
        </div>
      </div>
    </div>
  );
}
