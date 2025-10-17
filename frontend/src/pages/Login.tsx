import { useState } from "react";
import { useNavigate, Link } from "react-router-dom";
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
            <label className="sr-only" htmlFor="email">Email</label>
            <div className="field">
              <span className="field__icon" aria-hidden>
                <svg width="20" height="20" viewBox="0 0 32 32" fill="currentColor">
                  <path d="M26.249 28H5.753a.756.756 0 0 1-.75-.83C5.338 23.752 8.898 20 16 20s10.662 3.752 10.997 7.17a.756.756 0 0 1-.75.83zM16 18c-3.201 0-5.997-2.778-5.997-7.25 0-3.85 2.421-6.75 5.997-6.75s5.997 2.9 5.997 6.75c0 4.472-2.796 7.25-5.997 7.25z" fillRule="evenodd"></path>
                </svg>
              </span>
              <input
                id="email"
                type="email"
                placeholder="Email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
                autoComplete="email"
              />
            </div>

            <label className="sr-only" htmlFor="password">Password</label>
            <div className="field">
              <span className="field__icon" aria-hidden>
                <svg width="20" height="20" viewBox="0 0 64 64" fill="currentColor">
                  <path d="M44.01 28.02h1a4.177 4.177 0 0 1 4 4.334v17.334a4.177 4.177 0 0 1-4 4.333h-26a4.177 4.177 0 0 1-4-4.333V32.354a4.177 4.177 0 0 1 4-4.333h1v-7.005c0-4.03.846-6.379 2.729-8.262 1.883-1.883 4.23-2.73 8.258-2.73h2.026c4.029 0 6.376.847 8.258 2.73 1.883 1.883 2.729 4.231 2.729 8.262zm-4-7.002c0-2.565-.539-4.06-1.737-5.258-1.198-1.198-2.691-1.737-5.255-1.737h-2.016c-2.564 0-4.057.539-5.255 1.737-1.198 1.199-1.737 2.693-1.737 5.258v7.003h16z" fillRule="evenodd"></path>
                </svg>
              </span>
              <input
                id="password"
                type="password"
                placeholder="Password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
                autoComplete="current-password"
              />
            </div>
            <div className="forgot-password">
              <Link to="/forgot">Don't remember your password?</Link>
            </div>
            <button type="submit">Log In</button>
          </form>
          {error && <p>{error}</p>}
        </div>
      </div>
    </div>
  );
}
