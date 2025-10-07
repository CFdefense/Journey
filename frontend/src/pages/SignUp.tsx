import { useState } from 'react';
import { useNavigate , Link} from "react-router-dom"; 
import "../styles/Signup.css";
import { apiSignUp } from "../api/signup";
import * as logic from "../logic/SignUpLogic";

export default function Login() {
 const [firstName, setFirstName] = useState(""); 
 const [lastName, setLastName] = useState(""); 
 const [email, setEmail] = useState(""); 
 const [password, setPassword] = useState("");
 const [confirmPassword, setConfirmPassword] = useState("");
 const [error, setError] = useState(""); 
 const navigate = useNavigate();

 const handleSubmit = async (e: React.FormEvent) => {
      e.preventDefault(); // stops the page from refreshing 
      console.log(email, password, firstName, lastName);

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
      const result = await apiSignUp({
        email,
        first_name: firstName, // rust backend expects snake case as json variable
        last_name: lastName,
        password,
      });

        console.log("Account creation successful:", result);
        setError("");
        navigate("/create");
      } catch (err: any) {
        setError(err.message || "Sign Up failed.");
      }
  };
      
  return (
    <div className="signup-container">
      <div className="signup-box">
        <h1>Create Account</h1>
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
            First Name:
            <input
              type="text"
              value={firstName}
              onChange={(e) => setFirstName(e.target.value)}
              required
            />
          </label>
          <label>
            Last Name:
            <input
              type="text"
              value={lastName}
              onChange={(e) => setLastName(e.target.value)}
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

          <label>
            Re-Enter Password:
            <input
              type="password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              required
            />
          </label>

          <button type="submit">Create Account</button>
        </form>
        {error && <p style={{ color: "red" }}>{error}</p>}

        <div className="signup-actions">
        <Link to="/login" className="back-to-login-button">
            Already Have An Account?
        </Link>
      </div>
      
      </div>
    </div>
  );
}


