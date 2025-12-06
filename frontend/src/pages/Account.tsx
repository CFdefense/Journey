import { apiUpdateAccount, apiCurrent } from "../api/account";
import { useNavigate, useLocation } from "react-router-dom";
import { useState, useEffect } from "react";
import type { UpdateRequest } from "../models/account";
import Navbar from "../components/Navbar";
import "../styles/Account.css";
import userPfp from "../assets/user-pfp-temp.png";
import {
  checkIfValidPassword,
  checkIfPasswordsMatch,
  checkIfValidName
} from "../helpers/account";
import { toast } from "../components/Toast";

export default function Account() {
  const navigate = useNavigate();
  const location = useLocation();
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

  // Use same profile picture asset as Navbar for consistency
  const navbarAvatarUrl = userPfp;
  const [profileImageUrl, setProfileImageUrl] =
    useState<string>(navbarAvatarUrl);
  const [isEditingFirst, setIsEditingFirst] = useState<boolean>(false);
  const [isEditingLast, setIsEditingLast] = useState<boolean>(false);
  const [showPassword, setShowPassword] = useState<boolean>(false);
  const [tripsPlanned, setTripsPlanned] = useState<number | null>(null);
  const [accountCreated, setAccountCreated] = useState<string | null>(null);
  const [loaded, setLoaded] = useState<boolean>(false);

  const formatDate = (dateInput: string | number | Date): string => {
    const date = new Date(dateInput);
    if (Number.isNaN(date.getTime())) return "";
    return date.toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
      year: "numeric"
    });
  };

  // Fetch user profile on component mount
  useEffect(() => {
    async function fetchProfile() {
      const currentResult = await apiCurrent();

      if (currentResult.status == 401) {
        toast.error("You are not logged in. Please log in and try again.");
        navigate("/login");
        return;
      }

      if (currentResult.status == 404) {
        toast.error("Account not found. Please try again.");
        return;
      }

      const account = currentResult.result;
      if (account === null || currentResult.status !== 200) {
        toast.error("Failed to load account details. Please try again.");
        return;
      }

      setEmail(account.email || "");
      setFirstName(account.first_name || "");
      setLastName(account.last_name || "");
      // Optional stats from backend; otherwise, provide demo values
      const maybeTrips = (account as any).trips_planned;
      setTripsPlanned(typeof maybeTrips === "number" ? maybeTrips : 5);
      const maybeCreated = (account as any).created_at;
      setAccountCreated(
        maybeCreated ? formatDate(maybeCreated) : formatDate(new Date())
      );
      setLoaded(true);
    }

    fetchProfile();
  }, []);
  // Core submit/update logic used by form submit and inline "Done" buttons
  const submitUpdate = async () => {
    setPasswordErrors({});

    // Validate name fields
    const trimmedFirst = firstName.trim();
    const trimmedLast = lastName.trim();
    const nameError = checkIfValidName(trimmedFirst, trimmedLast);
    if (nameError) {
      toast.error(nameError);
      return;
    }

    // Check if user is trying to change password
    const isChangingPassword =
      currentPassword || newPassword || confirmPassword;

    if (isChangingPassword) {
      // Validate current password is provided
      if (!currentPassword) {
        setPasswordErrors({
          current: "Current password is required to change your password."
        });
        toast.error("Please provide your current password to change it.");
        return;
      }

      // Validate new password is provided
      if (!newPassword) {
        setPasswordErrors({ new: "New password is required." });
        toast.error("Please enter a new password.");
        return;
      }

      // Validate new password meets requirements
      const passwordValidationError = checkIfValidPassword(newPassword);
      if (passwordValidationError) {
        setPasswordErrors({ new: passwordValidationError });
        toast.error(passwordValidationError);
        return;
      }

      // Validate passwords match
      const matchError = checkIfPasswordsMatch(newPassword, confirmPassword);
      if (matchError) {
        setPasswordErrors({ confirm: matchError });
        toast.error(matchError);
        return;
      }
    }

    const payload: UpdateRequest = {
      email: null,
      first_name: trimmedFirst.length > 0 ? trimmedFirst : null,
      last_name: trimmedLast.length > 0 ? trimmedLast : null,
      password: isChangingPassword ? newPassword : null,
      current_password: isChangingPassword ? currentPassword : null,
      budget_preference: null,
      risk_preference: null,
      food_allergies: null,
      disabilities: null
    };

    const updateResult = await apiUpdateAccount(payload);

    if (updateResult.status == 401) {
      toast.error("Unauthorized user, please log in again.");
      navigate("/login");
    }

    if (!updateResult || updateResult.status !== 200) {
      // Handle password-related errors (400 Bad Request)
      if (updateResult.status === 400 && isChangingPassword) {
        toast.error("Current password is incorrect. Please try again.");
        setPasswordErrors({ current: "Current password is incorrect." });
      } else {
        toast.error("Update failed. Please try again.");
      }
      return;
    }

    toast.success(
      isChangingPassword
        ? "Password updated successfully!"
        : "Account updated successfully!"
    );

    // Update UI with any returned account info
    if (updateResult.result) {
      setFirstName(updateResult.result.first_name || trimmedFirst);
      setLastName(updateResult.result.last_name || trimmedLast);
    }

    // Clear password fields
    setCurrentPassword("");
    setNewPassword("");
    setConfirmPassword("");
    setPasswordErrors({});

    toast.error("Unable to update account. Please try again.");
  };

  const handleUpdate = async (e: React.FormEvent) => {
    e.preventDefault();
    await submitUpdate();
  };

  return (
    <div className="auth-page auth-page--account auth-page--no-scroll">
      <Navbar page="view" />

      <div className="auth-content">
        {loaded && (
          <div className="account-wrapper fade-in">
            {/* Main Content */}
            <main className="main-content">
              <div className="account-container">
                <div className="account-box">
                  <div className="hs-hero-card">
                    <div className="profile-header">
                      <div className="avatar-wrapper">
                        <img
                          src={profileImageUrl}
                          alt={`${firstName || "User"} ${lastName || ""}`.trim()}
                          className="avatar"
                          onError={() => setProfileImageUrl(navbarAvatarUrl)}
                        />
                      </div>
                      <div className="profile-meta">
                        <h1 className="profile-name">
                          {(firstName || "Your") + " " + (lastName || "Name")}
                        </h1>
                        <p className="profile-email">Account &amp; Settings</p>
                      </div>
                    </div>
                    <div className="hs-stats">
                      <div className="hs-stat">
                        <div className="hs-stat__value">
                          {tripsPlanned ?? 5}
                        </div>
                        <div className="hs-stat__label">Trips planned</div>
                      </div>
                      <div className="hs-stat">
                        <div className="hs-stat__value">
                          {accountCreated ?? formatDate(new Date())}
                        </div>
                        <div className="hs-stat__label">Account created</div>
                      </div>
                    </div>
                  </div>

                  <form onSubmit={handleUpdate}>
                    <div className="field-list">
                      <div className="field-row">
                        <div className="field-row__meta">
                          <div className="field-row__label">First name</div>
                          {isEditingFirst ? (
                            <input
                              type="text"
                              id="firstName"
                              value={firstName}
                              onChange={(e) => setFirstName(e.target.value)}
                            />
                          ) : (
                            <div className="field-row__value">
                              {firstName || "—"}
                            </div>
                          )}
                        </div>
                        <div className="field-row__action">
                          <button
                            type="button"
                            className="pill-button"
                            onClick={async () => {
                              if (isEditingFirst) {
                                await submitUpdate();
                              }
                              setIsEditingFirst(!isEditingFirst);
                            }}
                          >
                            {isEditingFirst ? "Done" : "Edit"}
                          </button>
                        </div>
                      </div>

                      <div className="field-row">
                        <div className="field-row__meta">
                          <div className="field-row__label">Last name</div>
                          {isEditingLast ? (
                            <input
                              type="text"
                              id="lastName"
                              value={lastName}
                              onChange={(e) => setLastName(e.target.value)}
                            />
                          ) : (
                            <div className="field-row__value">
                              {lastName || "—"}
                            </div>
                          )}
                        </div>
                        <div className="field-row__action">
                          <button
                            type="button"
                            className="pill-button"
                            onClick={async () => {
                              if (isEditingLast) {
                                await submitUpdate();
                              }
                              setIsEditingLast(!isEditingLast);
                            }}
                          >
                            {isEditingLast ? "Done" : "Edit"}
                          </button>
                        </div>
                      </div>

                      <div className="field-row">
                        <div className="field-row__meta">
                          <div className="field-row__label">Email address</div>
                          <div className="field-row__value">{email}</div>
                        </div>
                        <div className="field-row__action">
                          <button
                            type="button"
                            className="pill-button pill-button--disabled"
                            disabled
                            title="Cannot edit email associated with your account"
                          >
                            Edit
                          </button>
                        </div>
                      </div>
                    </div>

                    <div className="field-section">
                      <button
                        type="button"
                        className="field-section__header"
                        onClick={() => setShowPassword(!showPassword)}
                        aria-expanded={showPassword}
                      >
                        Password
                        <span
                          className={`chevron ${showPassword ? "up" : "down"}`}
                        ></span>
                      </button>
                      {showPassword && (
                        <div className="password-fields">
                          <div className="field-group">
                            <label htmlFor="currentPassword">
                              Current Password:
                            </label>
                            <input
                              type="password"
                              id="currentPassword"
                              value={currentPassword}
                              onChange={(e) =>
                                setCurrentPassword(e.target.value)
                              }
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
                                Password must be 8-128 characters, contain
                                uppercase, lowercase, and a number.
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
                              onChange={(e) =>
                                setConfirmPassword(e.target.value)
                              }
                              placeholder="Confirm new password"
                            />
                            {passwordErrors.confirm && (
                              <small className="error-message">
                                {passwordErrors.confirm}
                              </small>
                            )}
                          </div>

                          <div className="password-actions">
                            <button
                              type="button"
                              className="btn-primary"
                              onClick={submitUpdate}
                            >
                              Change password
                            </button>
                          </div>
                        </div>
                      )}
                    </div>
                  </form>
                </div>
              </div>
            </main>
          </div>
        )}
        {/* Bottom tab bar */}
        <footer className="account-bottom-bar">
          <div className="account-bottom-inner">
            <button
              type="button"
              className={`bottom-tab ${location.pathname === "/account" ? "active" : ""}`}
              onClick={() => navigate("/account")}
            >
              Account
            </button>
            <button
              type="button"
              className={`bottom-tab ${location.pathname.includes("/account/preferences") ? "active" : ""}`}
              onClick={() => navigate("/account/preferences")}
            >
              Preferences
            </button>
            <button
              type="button"
              className={`bottom-tab ${location.pathname.includes("/account/itineraries") ? "active" : ""}`}
              onClick={() => navigate("/account/itineraries")}
            >
              Itineraries
            </button>
          </div>
        </footer>
      </div>
    </div>
  );
}
