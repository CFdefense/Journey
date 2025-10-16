import type { PropsWithChildren } from "react";
import AuthNavbar from "./AuthNavbar";

type AuthLayoutProps = PropsWithChildren<{
  variant: "login" | "signup";
}>;

export default function AuthLayout({ variant, children }: AuthLayoutProps) {
  return (
    <div className={`auth-page auth-page--${variant}`}>
      <AuthNavbar variant={variant} />
      {children}
    </div>
  );
}


