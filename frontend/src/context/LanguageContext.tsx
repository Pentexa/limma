"use client";

import React, { createContext, useContext, useState, useEffect } from "react";
import { EN, TR, Translations } from "../i18n/locales";

type LanguageContextType = {
  lang: "EN" | "TR";
  t: Translations;
  toggleLanguage: () => void;
};

const LanguageContext = createContext<LanguageContextType | undefined>(undefined);

export function LanguageProvider({ children }: { children: React.ReactNode }) {
  const [lang, setLang] = useState<"EN" | "TR">("EN");

  useEffect(() => {
    const savedLang = localStorage.getItem("limma_lang") as "EN" | "TR";
    if (savedLang) setLang(savedLang);
  }, []);

  const toggleLanguage = () => {
    setLang((prev) => {
      const next = prev === "EN" ? "TR" : "EN";
      localStorage.setItem("limma_lang", next);
      return next;
    });
  };

  const t = lang === "EN" ? EN : TR;

  return (
    <LanguageContext.Provider value={{ lang, t, toggleLanguage }}>
      {children}
    </LanguageContext.Provider>
  );
}

export function useLanguage() {
  const context = useContext(LanguageContext);
  if (!context) throw new Error("useLanguage must be used within LanguageProvider");
  return context;
}
