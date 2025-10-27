import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import Index from "./pages/Index";
import Home from "./pages/Home";
import ViewItinerary from "./pages/ViewItinerary";
import Account from "./pages/Account";
import Preferences from './pages/Preferences';
import Login from "./pages/Login";
import Signup from "./pages/SignUp";
import Itineraries from './pages/Itineraries';
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
        <Route path="/" element={<Index />} />
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
        <Route 
          path="/account/preferences" 
          element={
            <ProtectedRoute>
              <Preferences />
            </ProtectedRoute>
          } 
        />
        <Route 
          path="/account/itineraries" 
          element={
            <ProtectedRoute>
              <Itineraries />
            </ProtectedRoute>
            
          } 
        />
        <Route path="*" element={<NotFound />} />
      </Routes>
    </Router>
  );
}
