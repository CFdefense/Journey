import { useNavigate } from "react-router-dom";
import { apiUpdateAccount, apiCurrent } from "../api/account";
import { useState, useEffect } from "react";
import type { UpdateRequest } from "../models/account";
import "../styles/Account.css";
import Navbar from "../components/Navbar";
import { useLocation } from "react-router-dom";
import { BudgetBucket, RiskTolerence } from "../models/account";
import userPfp from "../assets/user-pfp-temp.png";

type BudgetOption =
  | "VeryLowBudget"
  | "LowBudget"
  | "MediumBudget"
  | "HighBudget"
  | "LuxuryBudget";
type RiskOption = "ChillVibes" | "LightFun" | "Adventurer" | "RiskTaker";

function enumToString<T extends object>(
  enumType: T,
  enumValue: T[keyof T]
): string {
  // Enums in TypeScript allow lookup by value to get the key string.
  // We use keyof T to ensure type safety for the lookup.
  const key = Object.keys(enumType).find(
    (k) => enumType[k as keyof T] === enumValue
  );

  // If the key is found, return it. Otherwise, return a safe fallback or throw.
  // For numeric enums, the key is the string name.
  // For string enums, the key is the string value, which is what the API expects.
  // The .toString() call at the end ensures the return type is 'string' for the API.
  return key || (enumValue as any)?.toString() || "";
}

function stringToEnum<T extends object>(
  enumType: T,
  enumKey: string
): T[keyof T] | undefined {
  // We use `as keyof T` to cast the string key to an enum key for lookup.
  // This allows us to access the enum's value.
  const enumValue = enumType[enumKey as keyof T];

  // Check if the value is valid (i.e., not undefined)
  if (enumValue !== undefined) {
    return enumValue;
  }
  return undefined; // Return undefined if the key is not found
}

export default function Preferences() {
  const navigate = useNavigate();
  const location = useLocation();

  const [statusMessage, setStatusMessage] = useState<{
    type: "success" | "error";
    message: string;
  } | null>(null);
  const [loaded, setLoaded] = useState<boolean>(false);
  const [budget, setBudget] = useState<BudgetBucket>(BudgetBucket.MediumBudget);
  const [riskTolerance, setRiskTolerance] = useState<RiskTolerence>(
    RiskTolerence.LightFun
  );
  const [disabilities, setDisabilities] = useState("");
  const [foodPreferences, setFoodPreferences] = useState("");
  const [isEditingBudget, setIsEditingBudget] = useState<boolean>(false);
  const [isEditingRisk, setIsEditingRisk] = useState<boolean>(false);
  const [isEditingDisabilities, setIsEditingDisabilities] =
    useState<boolean>(false);
  const [isEditingFood, setIsEditingFood] = useState<boolean>(false);
  const [firstName, setFirstName] = useState<string>("");
  const [lastName, setLastName] = useState<string>("");
  const navbarAvatarUrl = userPfp;
  const [profileImageUrl, setProfileImageUrl] = useState<string>(navbarAvatarUrl);
  const [tripsPlanned, setTripsPlanned] = useState<number | null>(null);
  const [accountCreated, setAccountCreated] = useState<string | null>(null);
  const budgetOptions: BudgetOption[] = [
    "VeryLowBudget",
    "LowBudget",
    "MediumBudget",
    "HighBudget",
    "LuxuryBudget"
  ];
  const riskOptions: RiskOption[] = [
    "ChillVibes",
    "LightFun",
    "Adventurer",
    "RiskTaker"
  ];

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
    const fetchProfile = async () => {
      const { result, status } = await apiCurrent();

      if (status === 200 && result) {
        setFirstName(result.first_name || "");
        setLastName(result.last_name || "");
        setBudget(result.budget_preference as BudgetBucket);
        setRiskTolerance(result.risk_preference as RiskTolerence);
        setDisabilities(result.disabilities || "");
        setFoodPreferences(result.food_allergies || "");
        const maybeTrips = (result as any).trips_planned;
        setTripsPlanned(typeof maybeTrips === "number" ? maybeTrips : 5);
        const maybeCreated = (result as any).created_at;
        setAccountCreated(
          maybeCreated ? formatDate(maybeCreated) : formatDate(new Date())
        );
        setLoaded(true);
      } else {
        setStatusMessage({
          message: "Failed to load preferences. Please try again.",
          type: "error"
        });
        setLoaded(true);
      }
    };

    const fetchAccount = async () => {
      const currentResult = await apiCurrent();
      const account = currentResult.result;

      if (account && currentResult.status === 200) {
        setFirstName(account.first_name || "");
        setLastName(account.last_name || "");
        const maybeTrips = (account as any).trips_planned;
        setTripsPlanned(typeof maybeTrips === "number" ? maybeTrips : 5);
        const maybeCreated = (account as any).created_at;
        setAccountCreated(
          maybeCreated ? formatDate(maybeCreated) : formatDate(new Date())
        );
        setLoaded(true);
      }
    };

    fetchAccount();
    fetchProfile();
  }, []);

  // Legacy submit removed in favor of inline updates
  // Inline update for a single preference field
  const submitPartialUpdate = async (partial: Partial<UpdateRequest>) => {
    setStatusMessage(null);
    const payload: UpdateRequest = {
      email: null,
      first_name: null,
      last_name: null,
      password: null,
      current_password: null,
      budget_preference: null,
      risk_preference: null,
      disabilities: null,
      food_allergies: null,
      ...partial
    };
    const updateResult = await apiUpdateAccount(payload);
    if (updateResult.status !== 200) {
      setStatusMessage({
        message: "Update failed. Please try again.",
        type: "error"
      });
      return false;
    }
    setStatusMessage({
      message: "Preferences updated successfully!",
      type: "success"
    });
    return true;
  };

  return (
    <div className="auth-page auth-page--account auth-page--no-scroll">
      <Navbar page="view" firstName={firstName} />

      <div className="auth-content">
        <div className="account-wrapper">
          {/* Collapsible Sidebar */}
          <aside className="sidebar">
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
            {loaded && (
            <div className="account-container fade-in">
              <div className="account-box">
                <div className="hs-hero-card">
                  <div className="profile-header">
                    <div className="avatar-wrapper">
                      <img
                        src={profileImageUrl}
                        alt={`${firstName || "User"}`}
                        className="avatar"
                        onError={() => setProfileImageUrl(navbarAvatarUrl)}
                      />
                    </div>
                    <div className="profile-meta">
                      <h1 className="profile-name">
                        {`${firstName || ""} ${lastName || ""}`.trim() || "Your Name"}
                      </h1>
                      <p className="profile-email">Account Preferences</p>
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

                <div className="field-list">
                  {/* Budget */}
                  <div className={`field-row ${isEditingBudget ? "field-row--editing" : ""}`}>
                    <div className="field-row__meta">
                      <div className="field-row__label">Budget</div>
                      {isEditingBudget ? (
                        <select
                          id="budget"
                          value={enumToString(BudgetBucket, budget)}
                          onChange={(e) => {
                            const key = e.target.value;
                            const newVal = stringToEnum(BudgetBucket, key);
                            if (newVal !== undefined) {
                              setBudget(newVal as BudgetBucket);
                            }
                          }}
                        >
                          {budgetOptions.map((option) => (
                            <option key={option} value={option}>
                              {option}
                            </option>
                          ))}
                        </select>
                      ) : (
                        <div className="field-row__value">
                          {enumToString(BudgetBucket, budget)}
                        </div>
                      )}
                    </div>
                    <div className="field-row__action">
                      <button
                        type="button"
                        className="pill-button"
                        onClick={async () => {
                          if (isEditingBudget) {
                            await submitPartialUpdate({
                              budget_preference: budget
                            });
                          }
                          setIsEditingBudget(!isEditingBudget);
                        }}
                      >
                        {isEditingBudget ? "Done" : "Edit"}
                      </button>
                    </div>
                  </div>

                  {/* Risk Tolerance */}
                  <div className={`field-row ${isEditingRisk ? "field-row--editing" : ""}`}>
                    <div className="field-row__meta">
                      <div className="field-row__label">Risk tolerance</div>
                      {isEditingRisk ? (
                        <select
                          id="riskTolerance"
                          value={enumToString(RiskTolerence, riskTolerance)}
                          onChange={(e) => {
                            const key = e.target.value;
                            const newVal = stringToEnum(
                              RiskTolerence,
                              key
                            );
                            if (newVal !== undefined) {
                              setRiskTolerance(newVal as RiskTolerence);
                            }
                          }}
                        >
                          {riskOptions.map((option) => (
                            <option key={option} value={option}>
                              {option}
                            </option>
                          ))}
                        </select>
                      ) : (
                        <div className="field-row__value">
                          {enumToString(RiskTolerence, riskTolerance)}
                        </div>
                      )}
                    </div>
                    <div className="field-row__action">
                      <button
                        type="button"
                        className="pill-button"
                        onClick={async () => {
                          if (isEditingRisk) {
                            await submitPartialUpdate({
                              risk_preference: riskTolerance
                            });
                          }
                          setIsEditingRisk(!isEditingRisk);
                        }}
                      >
                        {isEditingRisk ? "Done" : "Edit"}
                      </button>
                    </div>
                  </div>

                  {/* Disabilities */}
                  <div className={`field-row ${isEditingDisabilities ? "field-row--editing" : ""}`}>
                    <div className="field-row__meta">
                      <div className="field-row__label">
                        Disabilities/Accessibility Needs
                      </div>
                      {isEditingDisabilities ? (
                        <textarea
                          id="disabilities"
                          value={disabilities}
                          onChange={(e) => setDisabilities(e.target.value)}
                          placeholder="e.g., Wheelchair user, visual impairment."
                        />
                      ) : (
                        <div className="field-row__value">
                          {disabilities || "—"}
                        </div>
                      )}
                    </div>
                    <div className="field-row__action">
                      <button
                        type="button"
                        className="pill-button"
                        onClick={async () => {
                          if (isEditingDisabilities) {
                            await submitPartialUpdate({
                              disabilities: disabilities || null
                            });
                          }
                          setIsEditingDisabilities(!isEditingDisabilities);
                        }}
                      >
                        {isEditingDisabilities ? "Done" : "Edit"}
                      </button>
                    </div>
                  </div>

                  {/* Food preferences */}
                  <div className={`field-row ${isEditingFood ? "field-row--editing" : ""}`}>
                    <div className="field-row__meta">
                      <div className="field-row__label">
                        Food Preferences/Allergies
                      </div>
                      {isEditingFood ? (
                        <textarea
                          id="foodPreferences"
                          value={foodPreferences}
                          onChange={(e) => setFoodPreferences(e.target.value)}
                          placeholder="e.g., Gluten-free, no shellfish, vegan."
                        />
                      ) : (
                        <div className="field-row__value">
                          {foodPreferences || "—"}
                        </div>
                      )}
                    </div>
                    <div className="field-row__action">
                      <button
                        type="button"
                        className="pill-button"
                        onClick={async () => {
                          if (isEditingFood) {
                            await submitPartialUpdate({
                              food_allergies: foodPreferences || null
                            });
                          }
                          setIsEditingFood(!isEditingFood);
                        }}
                      >
                        {isEditingFood ? "Done" : "Edit"}
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            )}
          </main>
        </div>
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
