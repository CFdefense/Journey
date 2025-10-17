import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import IndexPage from "./pages/IndexPage";
import Home from "./pages/Home";
import ViewItinerary from "./pages/ViewItinerary";
import Account from "./pages/Account";
import Login from "./pages/Login";
import Signup from "./pages/SignUp";
import NotFound from "./pages/NotFound";
import "./App.css";
import {
  InverseProtectedRoute,
  ProtectedRoute
} from "./components/ProtectedRoute";
import AuthLayout from "./components/AuthLayout";

export default function App() {
  return (
    <Router>
      <Routes>
        <Route path="/" element={<IndexPage />} />
        <Route
          path="/home"
          element={
            <ProtectedRoute>
              <Home />
            </ProtectedRoute>
          }
        />
        <Route
          path="/view"
          element={
            <ProtectedRoute>
              <ViewItinerary />
            </ProtectedRoute>
          }
        />
        <Route
          path="/account"
          element={
            <ProtectedRoute>
              <Account />
            </ProtectedRoute>
          }
        />
        <Route
          path="/login"
          element={
            <InverseProtectedRoute>
              <AuthLayout variant="login">
                <Login />
              </AuthLayout>
            </InverseProtectedRoute>
          }
        />
        <Route
          path="/signup"
          element={
            <AuthLayout variant="signup">
              <Signup />
            </AuthLayout>
          }
        />
        <Route path="*" element={<NotFound />} />
      </Routes>
    </Router>
  );
}
