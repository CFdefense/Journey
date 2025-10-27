import { Link } from "react-router-dom";
import { useContext, type Context } from "react";
import { GlobalContext } from "../helpers/global";
import type { GlobalState } from "./GlobalProvider";
import userPfp from "../assets/user-pfp-temp.png";

type NavbarProps = {
  page: "login" | "signup" | "index" | "home" | "view";
  firstName?: string;
};

export default function Navbar({ page, firstName }: NavbarProps) {
  const { authorized } = useContext<GlobalState>(
    GlobalContext as Context<GlobalState>
  );
  const renderCTA = () => {
    switch (page) {
      case "login":
        return (
          <div className="auth-cta">
            <span>Don't have an account?</span>
            <Link to="/signup" className="auth-cta-link">
              Sign up →
            </Link>
          </div>
        );
      case "signup":
        return (
          <div className="auth-cta">
            <span>Have an account?</span>
            <Link to="/login" className="auth-cta-link">
              Log in →
            </Link>
          </div>
        );
      case "index":
        if (authorized === true) {
          return (
            <div className="auth-cta">
              <Link to="/home" className="auth-cta-link">
                Create
              </Link>
              <Link to="/account" className="profile-pic-link">
                {/* Empty profile picture placeholder for now until we have a way to get the user's profile picture */}
              </Link>
            </div>
          );
        }
        return (
          <div className="auth-cta">
            <Link to="/signup" className="auth-cta-link">
              Sign up
            </Link>
            <Link to="/login" className="auth-cta-link">
              Log in
            </Link>
          </div>
        );
      case "home":
        return (
          <div className="auth-cta">
            <Link to="/account" className="auth-cta-link user-profile-link">
              <img src={userPfp} alt="User profile" className="user-profile-pic" />
              <span className="user-first-name">{firstName || "User"}</span>
            </Link>
          </div>
        );
      case "view":
        return (
          <div className="auth-cta">
            <Link to="/account" className="auth-cta-link user-profile-link">
              <img src={userPfp} alt="User profile" className="user-profile-pic" />
              <span className="user-first-name">{firstName || "User"}</span>
            </Link>
          </div>
        )
      default:
        return null;
    }
  };

  return (
    <header className="auth-navbar">
      <div className="auth-navbar-content">
        <Link to="/" className="auth-brand">
          Journey
        </Link>
        {renderCTA()}
      </div>
    </header>
  );
}