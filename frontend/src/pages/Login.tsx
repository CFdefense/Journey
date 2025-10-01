import { useState } from 'react';
import { useNavigate } from "react-router-dom"; 
import "./Login.css";

export default function Login() {
 const [userName, setUsername] = useState(""); // react hook to make sure that variable stays changed after React re-renders (gives components memory). https://react.dev/reference/react/useState
 const [password, setPassword] = useState("");
 const [error, setError] = useState(""); // for showing error messages
 const navigate = useNavigate();

 const handleSubmit = (e: React.FormEvent) => {
      e.preventDefault(); // stops the page from refreshing 
      console.log("username: " + userName + " password: " + password)

      // *****this will be replaced by backend validation API call
      if(userName === "123" && password === "123") {
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
            Username:
            <input
              type="text"
              value={userName}
              onChange={(e) => setUsername(e.target.value)}
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
      </div>
    </div>
  );
}

