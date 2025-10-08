import { BrowserRouter as Router, Routes, Route, Link } from "react-router-dom";
import IndexPage from "./pages/IndexPage";
import Home from "./pages/Home";
import ViewItinerary from "./pages/ViewItinerary";
import Account from "./pages/Account";
import Login from "./pages/Login";
import Signup from "./pages/SignUp";
import "./App.css";

function App() {
  return (
    <Router>
      {/* Navigation */}
      <nav>
        <Link to="/">Index</Link> | <Link to="/create">Home</Link> |{" "}
        <Link to="/view">View</Link> | <Link to="/account">Account</Link> |{" "}
        <Link to="/login">Login</Link> | <Link to="/signup">Signup</Link>
      </nav>

      {/* Routes */}
      <Routes>
        <Route path="/" element={<IndexPage />} />
        <Route path="/create" element={<Home />} />
        <Route path="/view" element={<ViewItinerary />} />
        <Route path="/account" element={<Account />} />
        <Route path="/login" element={<Login />} />
        <Route path="/signup" element={<Signup />} />
      </Routes>
    </Router>
  );
}

export default App;
