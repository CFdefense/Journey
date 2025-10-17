import { useState } from "react";
import { useNavigate } from "react-router-dom";
import "../styles/SignUp.css";
import { apiSignUp } from "../api/account";
import * as logic from "../helpers/account";

export default function Signup() {
  const [firstName, setFirstName] = useState("");
  const [lastName, setLastName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [error, setError] = useState("");
  const navigate = useNavigate();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    // sanitize user input
    const emailError = logic.checkIfValidEmail(email);
    if (emailError) {
      setError(emailError);
      return;
    }

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

    try {
      await apiSignUp({
        email,
        first_name: firstName, // rust backend expects snake case as json variable
        last_name: lastName,
        password
      });
      
      setError("");
      navigate("/home");
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : "Sign Up failed.";
      setError(message);
    }
  };

  return (
    <div className="auth-content">
      <div className="login-container">
        <div className="signup-page-content">
          <section className="signup-marketing">
            <div className="signup-brand">Journey</div>
            
            <p className="signup-sub">
              Create your account to start planning your next journey with the help of our team of intelligent AI agents.
            </p>
            <div className="signup-bullets">
              <div className="bullet-item">
                <div className="bullet-content">
                  <div className="bullet-title">Personalized AI Planning</div>
                  <div className="bullet-desc">Get personalized trip recommendations.</div>
                  <div className="bullet-desc">Fast, accurate, and tailored to your needs.</div>
                </div>
              </div>
              <div className="bullet-item">
                <div className="bullet-content">
                  <div className="bullet-title">Tailor itineraries together with collaborative chat</div>
                  <div className="bullet-desc">Work with AI to refine and customize your travel plans</div>
                  <div className="bullet-desc">Agents will grow with you and your preferences.</div>
                </div>
              </div>
              <div className="bullet-item">
                <div className="bullet-content">
                  <div className="bullet-title">Save and share your itineraries</div>
                  <div className="bullet-desc">Save and edit your plans with ease.</div>
                  <div className="bullet-desc">Share your itineraries with friends and family.</div>
                </div>
              </div>
            </div>
          </section>
          <div className="login-box signup-form-panel">
            <h1>Create your account</h1>
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
                First Name
                <input
                  type="text"
                  value={firstName}
                  onChange={(e) => setFirstName(e.target.value)}
                  required
                />
              </label>
              <label>
                Last Name
                <input
                  type="text"
                  value={lastName}
                  onChange={(e) => setLastName(e.target.value)}
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
              <label>
                Re-Enter Password
                <input
                  type="password"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  required
                />
              </label>
              <button type="submit">Get Started</button>
            </form>
            {error && <p>{error}</p>}
          </div>
        </div>
      </div>
    </div>
  );
}
