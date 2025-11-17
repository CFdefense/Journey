import { useContext, useState, type Context } from "react";
import { useNavigate, Link } from "react-router-dom";
import "../styles/SignUp.css";
import { apiSignUp } from "../api/account";
import * as logic from "../helpers/account";
import { GlobalContext } from "../helpers/global";
import type { GlobalState } from "../components/GlobalProvider";

export default function Signup() {
  const [firstName, setFirstName] = useState("");
  const [lastName, setLastName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);
  const [error, setError] = useState("");
  const navigate = useNavigate();
  const { setAuthorized } = useContext<GlobalState>(
    GlobalContext as Context<GlobalState>
  );

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    // sanitize user input
    const nameError = logic.checkIfValidName(firstName, lastName);
    if (nameError) {
      setError(nameError);
      return;
    }

    const passwordError = logic.checkIfValidPassword(password);
    if (passwordError) {
      setError(passwordError);
      return;
    }

    const matchError = logic.checkIfPasswordsMatch(password, confirmPassword);
    if (matchError) {
      setError(matchError);
      return;
    }

    const { status } = await apiSignUp({
      email,
      first_name: firstName, // rust backend expects snake case as json variable
      last_name: lastName,
      password
    });
    if (status === 409) {
      setError(
        "An account with this email already exists. Please log in instead."
      );
      return;
    }
    if (status !== 200) {
      setAuthorized(false);
      setError("Sign up failed.");
      return;
    }
    setAuthorized(true);
    console.log("Account creation successful");
    setError("");
    navigate("/home");
  };

  return (
    <div className="auth-content">
      <div className="login-container">
        <div className="signup-page-content">
          <section className="signup-marketing">
            <Link
              to="/home"
              className="signup-brand fade-in"
              style={{ animationDelay: "0ms" }}
            >
              Journey
            </Link>
            <p
              className="signup-sub fade-in"
              style={{ animationDelay: "50ms" }}
            >
              Create your account to start planning your next journey with the
              help of our team of intelligent AI agents.
            </p>
            <div className="signup-bullets">
              <div
                className="bullet-item fade-in"
                style={{ animationDelay: "100ms" }}
              >
                <div className="bullet-content">
                  <div className="bullet-title">Personalized AI Planning</div>
                  <div className="bullet-desc">
                    Get personalized trip recommendations.
                  </div>
                  <div className="bullet-desc">
                    Fast, accurate, and tailored to your needs.
                  </div>
                </div>
              </div>
              <div
                className="bullet-item fade-in"
                style={{ animationDelay: "150ms" }}
              >
                <div className="bullet-content">
                  <div className="bullet-title">
                    Tailor itineraries together with collaborative chat
                  </div>
                  <div className="bullet-desc">
                    Work with AI to refine and customize your travel plans
                  </div>
                  <div className="bullet-desc">
                    Agents will grow with you and your preferences.
                  </div>
                </div>
              </div>
              <div
                className="bullet-item fade-in"
                style={{ animationDelay: "200ms" }}
              >
                <div className="bullet-content">
                  <div className="bullet-title">
                    Save and share your itineraries
                  </div>
                  <div className="bullet-desc">
                    Save and edit your plans with ease.
                  </div>
                  <div className="bullet-desc">
                    Share your itineraries with friends and family.
                  </div>
                </div>
              </div>
            </div>
          </section>
          <div className="signup-box signup-form-panel">
            <h1 className="fade-in" style={{ animationDelay: "250ms" }}>
              Create your account
            </h1>
            <form onSubmit={handleSubmit}>
              <label className="sr-only" htmlFor="firstName">
                First Name
              </label>
              <div
                className="field fade-in"
                style={{ animationDelay: "300ms" }}
              >
                <span className="field__icon" aria-hidden>
                  <svg
                    width="20"
                    height="20"
                    viewBox="0 0 32 32"
                    fill="currentColor"
                  >
                    <path
                      d="M26.249 28H5.753a.756.756 0 0 1-.75-.83C5.338 23.752 8.898 20 16 20s10.662 3.752 10.997 7.17a.756.756 0 0 1-.75.83zM16 18c-3.201 0-5.997-2.778-5.997-7.25 0-3.85 2.421-6.75 5.997-6.75s5.997 2.9 5.997 6.75c0 4.472-2.796 7.25-5.997 7.25z"
                      fillRule="evenodd"
                    ></path>
                  </svg>
                </span>
                <input
                  id="firstName"
                  type="text"
                  placeholder="First Name"
                  value={firstName}
                  onChange={(e) => setFirstName(e.target.value)}
                  required
                  autoComplete="given-name"
                />
              </div>

              <label className="sr-only" htmlFor="lastName">
                Last Name
              </label>
              <div
                className="field fade-in"
                style={{ animationDelay: "350ms" }}
              >
                <span className="field__icon" aria-hidden>
                  <svg
                    width="20"
                    height="20"
                    viewBox="0 0 32 32"
                    fill="currentColor"
                  >
                    <path
                      d="M26.249 28H5.753a.756.756 0 0 1-.75-.83C5.338 23.752 8.898 20 16 20s10.662 3.752 10.997 7.17a.756.756 0 0 1-.75.83zM16 18c-3.201 0-5.997-2.778-5.997-7.25 0-3.85 2.421-6.75 5.997-6.75s5.997 2.9 5.997 6.75c0 4.472-2.796 7.25-5.997 7.25z"
                      fillRule="evenodd"
                    ></path>
                  </svg>
                </span>
                <input
                  id="lastName"
                  type="text"
                  placeholder="Last Name"
                  value={lastName}
                  onChange={(e) => setLastName(e.target.value)}
                  required
                  autoComplete="family-name"
                />
              </div>

              <label className="sr-only" htmlFor="email">
                Email
              </label>
              <div
                className="field fade-in"
                style={{ animationDelay: "400ms" }}
              >
                <span className="field__icon" aria-hidden>
                  <svg
                    width="20"
                    height="20"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                  >
                    <path d="M20 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z" />
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

              <label className="sr-only" htmlFor="password">
                Password
              </label>
              <div
                className="field field--password fade-in"
                style={{ animationDelay: "450ms" }}
              >
                <span className="field__icon" aria-hidden>
                  <svg
                    width="20"
                    height="20"
                    viewBox="0 0 64 64"
                    fill="currentColor"
                  >
                    <path
                      d="M44.01 28.02h1a4.177 4.177 0 0 1 4 4.334v17.334a4.177 4.177 0 0 1-4 4.333h-26a4.177 4.177 0 0 1-4-4.333V32.354a4.177 4.177 0 0 1 4-4.333h1v-7.005c0-4.03.846-6.379 2.729-8.262 1.883-1.883 4.23-2.73 8.258-2.73h2.026c4.029 0 6.376.847 8.258 2.73 1.883 1.883 2.729 4.231 2.729 8.262zm-4-7.002c0-2.565-.539-4.06-1.737-5.258-1.198-1.198-2.691-1.737-5.255-1.737h-2.016c-2.564 0-4.057.539-5.255 1.737-1.198 1.199-1.737 2.693-1.737 5.258v7.003h16z"
                      fillRule="evenodd"
                    ></path>
                  </svg>
                </span>
                <input
                  id="password"
                  type={showPassword ? "text" : "password"}
                  placeholder="Password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                  autoComplete="new-password"
                />
                <button
                  type="button"
                  className="password-toggle"
                  aria-label={showPassword ? "Hide password" : "Show password"}
                  onClick={() => setShowPassword(!showPassword)}
                >
                  {showPassword ? (
                    <svg
                      width="20"
                      height="20"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="2"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      aria-hidden="true"
                    >
                      <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path>
                      <line x1="1" y1="1" x2="23" y2="23"></line>
                    </svg>
                  ) : (
                    <svg
                      width="20"
                      height="20"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="2"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      aria-hidden="true"
                    >
                      <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
                      <circle cx="12" cy="12" r="3"></circle>
                    </svg>
                  )}
                </button>
              </div>

              <label className="sr-only" htmlFor="confirmPassword">
                Re-Enter Password
              </label>
              <div
                className="field field--password fade-in"
                style={{ animationDelay: "500ms" }}
              >
                <span className="field__icon" aria-hidden>
                  <svg
                    width="20"
                    height="20"
                    viewBox="0 0 64 64"
                    fill="currentColor"
                  >
                    <path
                      d="M44.01 28.02h1a4.177 4.177 0 0 1 4 4.334v17.334a4.177 4.177 0 0 1-4 4.333h-26a4.177 4.177 0 0 1-4-4.333V32.354a4.177 4.177 0 0 1 4-4.333h1v-7.005c0-4.03.846-6.379 2.729-8.262 1.883-1.883 4.23-2.73 8.258-2.73h2.026c4.029 0 6.376.847 8.258 2.73 1.883 1.883 2.729 4.231 2.729 8.262zm-4-7.002c0-2.565-.539-4.06-1.737-5.258-1.198-1.198-2.691-1.737-5.255-1.737h-2.016c-2.564 0-4.057.539-5.255 1.737-1.198 1.199-1.737 2.693-1.737 5.258v7.003h16z"
                      fillRule="evenodd"
                    ></path>
                  </svg>
                </span>
                <input
                  id="confirmPassword"
                  type={showConfirmPassword ? "text" : "password"}
                  placeholder="Re-Enter Password"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  required
                  autoComplete="new-password"
                />
                <button
                  type="button"
                  className="password-toggle"
                  aria-label={
                    showConfirmPassword ? "Hide password" : "Show password"
                  }
                  onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                >
                  {showConfirmPassword ? (
                    <svg
                      width="20"
                      height="20"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="2"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      aria-hidden="true"
                    >
                      <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path>
                      <line x1="1" y1="1" x2="23" y2="23"></line>
                    </svg>
                  ) : (
                    <svg
                      width="20"
                      height="20"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="2"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      aria-hidden="true"
                    >
                      <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
                      <circle cx="12" cy="12" r="3"></circle>
                    </svg>
                  )}
                </button>
              </div>

              <button
                type="submit"
                className="fade-in"
                style={{ animationDelay: "550ms" }}
              >
                Get Started
              </button>
            </form>
            {error && <p>{error}</p>}
          </div>
        </div>
      </div>
    </div>
  );
}
