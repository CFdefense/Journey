import {AUTH_TOKEN_LOCAL} from "./Globals"
import {useNavigate} from "react-router-dom";

export default function Account() {
  const navigate = useNavigate();

  const onLogout = async () => {
    console.log("Logging out");
    localStorage.removeItem(AUTH_TOKEN_LOCAL);
    navigate("/");
  }

  return (
    <div>
      <h1>Account Page</h1>
	  <button onClick={onLogout}>Logout</button>
    </div>
  );
}