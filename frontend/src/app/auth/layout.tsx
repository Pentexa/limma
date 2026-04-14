import type { Metadata } from "next";
import "@/styles/globals.css";

export const metadata: Metadata = {
  title: "LIMMA — Authentication",
  description: "Sign in or create an account for the LIMMA Security Intelligence Platform.",
};

export default function AuthLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <div className="auth-layout">
      {children}
    </div>
  );
}
