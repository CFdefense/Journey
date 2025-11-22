import { Link, useLocation } from "react-router-dom";
import { useContext, useEffect, useRef, useState, type Context } from "react";
import { GlobalContext } from "../helpers/global";
import type { GlobalState } from "./GlobalProvider";
import userPfp from "../assets/user-pfp-temp.png";
import { apiLogout } from "../api/account";
import { apiCurrent } from "../api/account";
import { ACTIVE_CHAT_SESSION } from "../pages/Home";
import "../styles/Navbar.css";

type NavbarProps = {
  page: "login" | "signup" | "index" | "home" | "view";
  firstName?: string;
  profileImageUrl?: string;  // Add this
};

export default function Navbar({ page, firstName, profileImageUrl }: NavbarProps) {
  const location = useLocation();
  const isAccountRoute = location.pathname.startsWith("/account");
  const { authorized, setAuthorized } = useContext<GlobalState>(
    GlobalContext as Context<GlobalState>
  );
  const [menuOpen, setMenuOpen] = useState<boolean>(false);
  const menuRef = useRef<HTMLDivElement | null>(null);
  const [displayName, setDisplayName] = useState<string>(firstName || "");
  const [avatarUrl, setAvatarUrl] = useState<string>(profileImageUrl || userPfp);  // Add this
  const nameReady = (displayName || "").trim().length > 0;

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setMenuOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  useEffect(() => {
    // Update avatar when prop changes
    if (profileImageUrl) {
      setAvatarUrl(profileImageUrl);
    }
  }, [profileImageUrl]);

  useEffect(() => {
    // Prefer explicit prop when provided
    if (firstName && firstName.trim().length > 0) {
      setDisplayName(firstName);
    }
    // Fallback: when authorized, fetch current user's first name and profile picture
    if (authorized && (!firstName || !profileImageUrl)) {
      apiCurrent()
        .then((res) => {
          const name = (res?.result?.first_name as string) || "";
          const picture = (res?.result?.profile_picture as string) || userPfp;
          if (!firstName) setDisplayName(name);
          if (!profileImageUrl) setAvatarUrl(picture);
        })
        .catch(() => {
          setDisplayName("");
          setAvatarUrl(userPfp);
        });
    } else if (!authorized) {
      setDisplayName("");
      setAvatarUrl(userPfp);
    }
  }, [authorized, firstName, profileImageUrl]);

  const onLogout = async () => {
    const { status } = await apiLogout();
    if (status !== 200) {
      console.error("Logout failed with status", status);
    }
    setAuthorized(false);
    sessionStorage.removeItem(ACTIVE_CHAT_SESSION);
    setMenuOpen(false);
  };

  const UserMenu = () => {
    return (
      <div className="user-menu" ref={menuRef}>
        <button
          type="button"
          className="user-menu-button"
          onClick={() => setMenuOpen((v) => !v)}
          aria-expanded={menuOpen}
          aria-haspopup="menu"
        >
          <span
            className={`user-menu-name ${displayName ? "ready" : "pending"}`}
          >
            {displayName || ""}
          </span>
          <img 
            src={avatarUrl} 
            alt="User profile" 
            className="user-menu-avatar"
            onError={(e) => { e.currentTarget.src = userPfp; }}
          />
        </button>
        <div
          className={`user-menu-dropdown ${menuOpen ? "open" : ""}`}
          role="menu"
          aria-hidden={!menuOpen}
        >
          <Link
            to="/account"
            className="user-menu-item"
            role="menuitem"
            onClick={() => setMenuOpen(false)}
          >
            Account
          </Link>
          <button
            type="button"
            className="user-menu-item user-menu-item--danger"
            role="menuitem"
            onClick={onLogout}
          >
            Log out
          </button>
        </div>
      </div>
    );
  };
  
  const renderCTA = () => {
    switch (page) {
      case "login":
        return (
          <div className="auth-cta">
            <span>Don't have an account?</span>
            <Link to="/signup" className="auth-cta-link">
              Sign up <span className="arrow">→</span>
            </Link>
          </div>
        );
      case "signup":
        return (
          <div className="auth-cta">
            <span>Have an account?</span>
            <Link to="/login" className="auth-cta-link">
              Log in <span className="arrow">→</span>
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
              <UserMenu />
            </div>
          );
        }
        if (authorized === null) {
          // Reserve space to avoid flashing between login/signup and user menu
          return (
            <div className="auth-cta auth-cta--pending" aria-hidden="true" />
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
            <UserMenu />
          </div>
        );
      case "view":
        return (
          <div className="auth-cta">
            {isAccountRoute && (
              <Link to="/home" className="auth-cta-link">
                Create
              </Link>
            )}
            <UserMenu />
          </div>
        );
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