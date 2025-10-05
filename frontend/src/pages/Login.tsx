import { useState } from 'react';
import { useNavigate , Link} from "react-router-dom"; 
import "./Login.css";

export default function Login() {
 const [email, setEmail] = useState(""); // react hook to make sure that variable stays changed after React re-renders (gives components memory). https://react.dev/reference/react/useState
 const [password, setPassword] = useState("");
 const [error, setError] = useState(""); // for showing error messages
 const navigate = useNavigate();

 const handleSubmit = (e: React.FormEvent) => {
      e.preventDefault(); // stops the page from refreshing 
      console.log("email: " + email + " password: " + password)

      // *****this will be replaced by backend validation API call
      if(email === "123@gmail.com" && password === "123") {
        setError("");
        navigate("/create") // go to itinerary creation page by default
      } else {
        setError("Invalid username or password.")
      }
      
 }
 
  return (
    <div className="login-container">
      <div className="login-box">
        <h1>Login Page</h1>
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

