import { Link } from "react-router-dom";

type AuthNavbarProps = {
  variant: "login" | "signup";
};

export default function AuthNavbar({ variant }: AuthNavbarProps) {
  return (
    <header className="auth-navbar">
      <div className="auth-navbar-content">
        <Link to="/" className="auth-brand">Journey</Link>

        {variant === "login" ? (
          <div className="auth-cta">
            <span>Don't have an account?</span>
            <Link to="/signup" className="auth-cta-link">
              Sign up
            </Link>
          </div>
        ) : (
          <div className="auth-cta"><span>Have an account?</span><Link to="/login" className="auth-cta-link">Log in →</Link></div>
        )}
      </div>
    </header>
  );
}


