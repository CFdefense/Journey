import { useState } from 'react';
import { useNavigate , Link} from "react-router-dom"; 
import "../styles/Signup.css";
import { apiSignUp } from "../api/signup";

export default function Login() {
 const [firstName, setFirstName] = useState(""); 
 const [lastName, setLastName] = useState(""); 
 const [email, setEmail] = useState(""); 
 const [password, setPassword] = useState("");
 const [password2, setPassword2] = useState("");
 const [error, setError] = useState(""); 
 const navigate = useNavigate();

 const handleSubmit = async (e: React.FormEvent) => {
      e.preventDefault(); // stops the page from refreshing 
      console.log(email, password, firstName, lastName);

      try {
        const result = await apiSignUp({email, firstName, lastName, password});
        console.log("Account Creation successful: " + result);
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
              value={password2}
              onChange={(e) => setPassword2(e.target.value)}
              required
            />
          </label>

          <button type="submit">Create Account</button>
        </form>
        {error && <p style={{ color: "red" }}>{error}</p>}
      
      </div>
    </div>
  );
}


