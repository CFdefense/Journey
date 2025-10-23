import type { PropsWithChildren } from "react";
import Navbar from "./Navbar";

type AuthLayoutProps = PropsWithChildren<{
  variant: "login" | "signup";
}>;

export default function AuthLayout({ variant, children }: AuthLayoutProps) {
  return (
    <div className={`auth-page auth-page--${variant}`}>
      <Navbar page={variant} />
      {children}
    </div>
  );
}
