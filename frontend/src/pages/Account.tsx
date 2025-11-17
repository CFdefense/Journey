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

export default function Account() {

  const navigate = useNavigate();
  const location = useLocation();
  const [statusMessage, setStatusMessage] = useState<{
    type: "success" | "error";
    message: string;
  } | null>(null);
  const [email, setEmail] = useState("");
  const [firstName, setFirstName] = useState<string>("");
  const [lastName, setLastName] = useState<string>("");
  //const [profile_picture, setProfilePicture] = useState<string>("");
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
  const [profileImageUrl, setProfileImageUrl] = useState<string>(navbarAvatarUrl);
  const [newProfilePicture, setNewProfilePicture] = useState<string>("")
  const [isEditingFirst, setIsEditingFirst] = useState<boolean>(false);
  const [isEditingLast, setIsEditingLast] = useState<boolean>(false);
  const [showPassword, setShowPassword] = useState<boolean>(false);
  const [tripsPlanned, setTripsPlanned] = useState<number | null>(null);
  const [accountCreated, setAccountCreated] = useState<string | null>(null);
  const [loaded, setLoaded] = useState<boolean>(false);
  const [showProfilePicModal, setShowProfilePicModal] = useState<boolean>(false);

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
      setProfileImageUrl(account.profile_picture || navbarAvatarUrl);
      // Optional stats from backend; otherwise, provide demo values
      const maybeTrips = (account as any).trips_planned;
      setTripsPlanned(
        typeof maybeTrips === "number" ? maybeTrips : 5
      );
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
    setStatusMessage(null);
    setPasswordErrors({});

    // Validate name fields
    const trimmedFirst = firstName.trim();
    const trimmedLast = lastName.trim();
    const nameError = checkIfValidName(trimmedFirst, trimmedLast);
    if (nameError) {
      setStatusMessage({
        message: nameError,
        type: "error"
      });
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
      first_name: trimmedFirst.length > 0 ? trimmedFirst : null,
      last_name: trimmedLast.length > 0 ? trimmedLast : null,
      password: isChangingPassword ? newPassword : null,
      current_password: isChangingPassword ? currentPassword : null,
      budget_preference: null,
      risk_preference: null,
      food_allergies: null,
      disabilities: null,
      profile_picture: newProfilePicture.trim().length > 0 ? newProfilePicture.trim() : null
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

    // Update UI with any returned account info
    if (updateResult.result) {
      setFirstName(updateResult.result.first_name || trimmedFirst);
      setLastName(updateResult.result.last_name || trimmedLast);
      if (updateResult.result.profile_picture) {
        setProfileImageUrl(updateResult.result.profile_picture);
      }
      setNewProfilePicture("");
    }

    // Clear password fields
    setCurrentPassword("");
    setNewPassword("");
    setConfirmPassword("");
    setPasswordErrors({});
  };

  const handleUpdate = async (e: React.FormEvent) => {
    e.preventDefault();
    await submitUpdate();
  };

  return (
    <div className="auth-page auth-page--account auth-page--no-scroll">
      <Navbar page="view" firstName={firstName} profileImageUrl={profileImageUrl} />
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
                      <button
                        type="button"
                        className="avatar-edit-button"
                        onClick={() => {
                          setNewProfilePicture(profileImageUrl === navbarAvatarUrl ? "" : profileImageUrl);
                          setShowProfilePicModal(true);
                        }}
                        aria-label="Edit profile picture"
                      >
                        ✒️
                      </button>
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

                {statusMessage && (
                  <div
                    className={`status-message status-message--${statusMessage.type}`}
                  >
                    {statusMessage.message}
                  </div>
                )}

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
                      <span className={`chevron ${showPassword ? "up" : "down"}`}></span>
                    </button>
                    {showPassword && (
                      <div className="password-fields">
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
                {showProfilePicModal && (
                  <div className="modal-overlay" onClick={() => {
                    setShowProfilePicModal(false);
                    setNewProfilePicture("");
                  }}>
                    <div className="modal-content" onClick={(e) => e.stopPropagation()}>
                      <h2>Change Profile Picture</h2>
                      
                      {/* File input */}
                      <input
                        type="file"
                        accept="image/*"
                        onChange={(e) => {
                          const file = e.target.files?.[0];
                          if (file) {
                            // Check file size (limit to 5MB)
                            if (file.size > 5 * 1024 * 1024) {
                              setStatusMessage({
                                type: "error",
                                message: "Image must be smaller than 5MB"
                              });
                              return;
                            }
                            
                            const reader = new FileReader();
                            reader.onload = () => {
                              setNewProfilePicture(reader.result as string);
                            };
                            reader.readAsDataURL(file);
                          }
                        }}
                        className="modal-file-input"
                      />
                      
                      {/* Preview */}
                      {newProfilePicture && (
                        <div className="preview-container">
                          <img 
                            src={newProfilePicture} 
                            alt="Preview" 
                            className="preview-image"
                          />
                        </div>
                      )}
                      
                      <div className="modal-actions">
                        <button 
                          className="modal-btn modal-btn--secondary"
                          onClick={() => {
                            setShowProfilePicModal(false);
                            setNewProfilePicture("");
                          }}
                        >
                          Cancel
                        </button>
                        <button 
                          className="modal-btn modal-btn--primary"
                          onClick={async () => {
                            await submitUpdate();
                            setShowProfilePicModal(false);
                          }}
                          disabled={!newProfilePicture}
                        >
                          Save
                        </button>
                      </div>
                    </div>
                  </div>
                )}
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
