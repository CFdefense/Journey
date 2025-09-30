import { useState } from 'react';
import "./Login.css";

export default function Login() {
 const [userName, setUsername] = useState(""); // react hook to make sure that variable stays changed after React re-renders (gives components memory). https://react.dev/reference/react/useState
 const [password, setPassword] = useState("");

 const handleSubmit = (e: React.FormEvent) => {
      e.preventDefault(); // stops the page from refreshing 
      console.log("username: " + userName + " password: " + password)
      // ******** this is where it will get sent to the backend
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
      </div>
    </div>
  );
}

