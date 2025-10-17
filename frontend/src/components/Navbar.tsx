import { Link } from "react-router-dom";

type NavbarProps = {
  page: "login" | "signup";
};

export default function Navbar({ page }: NavbarProps) {
  const renderCTA = () => {
    switch (page) {
      case "login":
        return (
          <div className="auth-cta">
            <span>Don't have an account?</span>
            <Link to="/signup" className="auth-cta-link">
              Sign up
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
      default:
        return null;
    }
  };

  return (
    <header className="auth-navbar">
      <div className="auth-navbar-content">
        <Link to="/" className="auth-brand">Journey</Link>
        {renderCTA()}
      </div>
    </header>
  );
}

