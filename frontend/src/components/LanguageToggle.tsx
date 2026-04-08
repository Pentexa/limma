"use client";

import { useLanguage } from "../context/LanguageContext";
import { Languages } from "lucide-react";

export function LanguageToggle() {
  const { lang, toggleLanguage } = useLanguage();

  return (
    <button
      onClick={toggleLanguage}
      className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-white/5 border border-white/10 hover:bg-white/10 transition-colors text-xs font-medium text-gray-300 w-full justify-center mt-2 group"
    >
      <Languages className="h-4 w-4 text-gray-400 group-hover:text-accent-cyan transition-colors" />
      <span>{lang === "EN" ? "Türkçe'ye Geç" : "Switch to English"}</span>
    </button>
  );
}
