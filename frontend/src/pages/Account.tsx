import { useNavigate } from "react-router-dom";
import { apiLogout, apiUpdateAccount, apiCurrent } from "../api/account";
import { useContext, useState, useEffect, type Context } from "react";
import { GlobalContext } from "../helpers/global";
import type { GlobalState } from "../components/GlobalProvider";
import type { UpdateRequest } from "../models/account";
import Navbar from "../components/Navbar";
import "../styles/Account.css";
import { ACTIVE_CHAT_SESSION } from "./Home";
import {
  checkIfValidPassword,
  checkIfPasswordsMatch
} from "../helpers/account";

export default function Account() {
  const { setAuthorized } = useContext<GlobalState>(
    GlobalContext as Context<GlobalState>
  );
  const navigate = useNavigate();

  const [statusMessage, setStatusMessage] = useState<{
    type: "success" | "error";
    message: string;
  } | null>(null);
  const [email, setEmail] = useState("");
  const [firstName, setFirstName] = useState<string>("");
  const [lastName, setLastName] = useState<string>("");
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [passwordErrors, setPasswordErrors] = useState<{
    current?: string;
    new?: string;
    confirm?: string;
  }>({});

  // Fetch user profile on component mount
  useEffect(() => {
    async function fetchProfile() {
      const currentResult = await apiCurrent();
      // TODO 401 -> navigate to /login

      const account = currentResult.result;
      if (account === null || currentResult.status !== 200) {
        console.error(
          "API call to /api/account/current failed with status: ",
          currentResult.status
        );
        setStatusMessage({
          message: "Failed to load account details. Please log in again.",
          type: "error"
        });
        return;
      }

      setEmail(account.email || "");
      setFirstName(account.first_name || "");
      setLastName(account.last_name || "");
    }

    fetchProfile();
  }, []);
  const onLogout = async () => {
    console.log("Logging out");
    const { status } = await apiLogout();
    if (status !== 200) {
      console.error("Logout failed with status", status);
    }
    setAuthorized(false);
    sessionStorage.removeItem(ACTIVE_CHAT_SESSION);
  };

  const handleUpdate = async (e: React.FormEvent) => {
    e.preventDefault();
    setStatusMessage(null);
    setPasswordErrors({});

    // Check if user is trying to change password
    const isChangingPassword =
      currentPassword || newPassword || confirmPassword;

    if (isChangingPassword) {
      // Validate current password is provided
      if (!currentPassword) {
        setPasswordErrors({
          current: "Current password is required to change your password."
        });
        setStatusMessage({
          message: "Please provide your current password to change it.",
          type: "error"
        });
        return;
      }

      // Validate new password is provided
      if (!newPassword) {
        setPasswordErrors({ new: "New password is required." });
        setStatusMessage({
          message: "Please enter a new password.",
          type: "error"
        });
        return;
      }

      // Validate new password meets requirements
      const passwordValidationError = checkIfValidPassword(newPassword);
      if (passwordValidationError) {
        setPasswordErrors({ new: passwordValidationError });
        setStatusMessage({
          message: passwordValidationError,
          type: "error"
        });
        return;
      }

      // Validate passwords match
      const matchError = checkIfPasswordsMatch(newPassword, confirmPassword);
      if (matchError) {
        setPasswordErrors({ confirm: matchError });
        setStatusMessage({
          message: matchError,
          type: "error"
        });
        return;
      }
    }

    const payload: UpdateRequest = {
      email: null,
      first_name: null,
      last_name: null,
      password: isChangingPassword ? newPassword : null,
      current_password: isChangingPassword ? currentPassword : null,
      budget_preference: null,
      risk_preference: null,
      food_allergies: null,
      disabilities: null
    };

    const updateResult = await apiUpdateAccount(payload);

    if (updateResult.status !== 200) {
      console.error(
        "API call to /api/account/update failed with status: ",
        updateResult.status
      );

      // Handle password-related errors (400 Bad Request)
      if (updateResult.status === 400 && isChangingPassword) {
        setStatusMessage({
          message: "Current password is incorrect. Please try again.",
          type: "error"
        });
        setPasswordErrors({ current: "Current password is incorrect." });
      } else {
        setStatusMessage({
          message: "Update failed. Please try again.",
          type: "error"
        });
      }
      return;
    }

    setStatusMessage({
      message: isChangingPassword
        ? "Password updated successfully!"
        : "Account settings updated successfully!",
      type: "success"
    });

    // Clear password fields
    setCurrentPassword("");
    setNewPassword("");
    setConfirmPassword("");
    setPasswordErrors({});
  };

  return (
    <div className="auth-page auth-page--account">
      <Navbar page="view" firstName={firstName} />

      <div className="auth-content">
        <div className="account-wrapper">
          {/* Collapsible Sidebar */}
          <aside className="sidebar">
            <div className="sidebar-toggle">
              <span></span>
              <span></span>
              <span></span>
            </div>

            <div className="sidebar-content">
              <button
                className="sidebar-item"
                onClick={() => navigate("/account")}
              >
                <div className="sidebar-icon">
                  <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                  >
                    <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                    <circle cx="12" cy="7" r="4" />
                  </svg>
                </div>
                <span className="sidebar-label">Account Information</span>
              </button>

              <button
                className="sidebar-item"
                onClick={() => navigate("/account/preferences")}
              >
                <div className="sidebar-icon">
                  <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                  >
                    <circle cx="12" cy="12" r="3" />
                    <path d="M12 1v6m0 6v6m9-9h-6m-6 0H3" />
                  </svg>
                </div>
                <span className="sidebar-label">Preference Update</span>
              </button>

              <button
                className="sidebar-item"
                onClick={() => navigate("/account/itineraries")}
              >
                <div className="sidebar-icon">
                  <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                  >
                    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                    <polyline points="14 2 14 8 20 8" />
                    <line x1="16" y1="13" x2="8" y2="13" />
                    <line x1="16" y1="17" x2="8" y2="17" />
                    <polyline points="10 9 9 9 8 9" />
                  </svg>
                </div>
                <span className="sidebar-label">Saved Itineraries</span>
              </button>
            </div>
          </aside>

          {/* Main Content */}
          <main className="main-content">
            <div className="account-container">
              <div className="account-box">
                <h1>Account Settings</h1>

                {statusMessage && (
                  <div
                    className={`status-message status-message--${statusMessage.type}`}
                  >
                    {statusMessage.message}
                  </div>
                )}

                <form onSubmit={handleUpdate}>
                  <div className="settings-section">
                    <h2>Account Information</h2>

                    <div className="field-group">
                      <label htmlFor="firstName">First Name:</label>
                      <input
                        type="text"
                        id="firstName"
                        value={firstName}
                        readOnly
                        className="input-readonly"
                      />
                    </div>

                    <div className="field-group">
                      <label htmlFor="lastName">Last Name:</label>
                      <input
                        type="text"
                        id="lastName"
                        value={lastName}
                        readOnly
                        className="input-readonly"
                      />
                    </div>

                    <div className="field-group">
                      <label htmlFor="email">Username (Email):</label>
                      <input
                        type="email"
                        id="email"
                        value={email}
                        readOnly
                        className="input-readonly"
                      />
                      <small>
                        Your username is linked to your email and cannot be
                        changed here.
                      </small>
                    </div>
                  </div>

                  <div className="settings-section">
                    <h2>Change Password</h2>

                    <div className="field-group">
                      <label htmlFor="currentPassword">Current Password:</label>
                      <input
                        type="password"
                        id="currentPassword"
                        value={currentPassword}
                        onChange={(e) => setCurrentPassword(e.target.value)}
                        placeholder="Enter your current password"
                      />
                      {passwordErrors.current && (
                        <small className="error-message">
                          {passwordErrors.current}
                        </small>
                      )}
                    </div>

                    <div className="field-group">
                      <label htmlFor="newPassword">New Password:</label>
                      <input
                        type="password"
                        id="newPassword"
                        value={newPassword}
                        onChange={(e) => setNewPassword(e.target.value)}
                        placeholder="Enter new password"
                      />
                      {passwordErrors.new && (
                        <small className="error-message">
                          {passwordErrors.new}
                        </small>
                      )}
                      {!passwordErrors.new && newPassword && (
                        <small className="helper-text">
                          Password must be 8-128 characters, contain uppercase,
                          lowercase, and a number.
                        </small>
                      )}
                    </div>

                    <div className="field-group">
                      <label htmlFor="confirmPassword">
                        Confirm New Password:
                      </label>
                      <input
                        type="password"
                        id="confirmPassword"
                        value={confirmPassword}
                        onChange={(e) => setConfirmPassword(e.target.value)}
                        placeholder="Confirm new password"
                      />
                      {passwordErrors.confirm && (
                        <small className="error-message">
                          {passwordErrors.confirm}
                        </small>
                      )}
                    </div>

                    <small className="helper-text">
                      Leave password fields blank if you don't want to change
                      your password.
                    </small>
                  </div>

                  <button type="submit" className="btn-primary">
                    Update Account Information
                  </button>
                </form>

                <hr className="section-divider" />

                <div className="logout-section">
                  <button onClick={onLogout} className="btn-danger">
                    Logout
                  </button>
                </div>
              </div>
            </div>
          </main>
        </div>
      </div>
    </div>
  );
}
