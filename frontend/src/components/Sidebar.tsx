"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import Image from "next/image";
import { motion } from "framer-motion";
import {
  ShieldAlert,
  Activity,
  Search,
  Network,
  Database,
  TerminalSquare,
  Fingerprint
} from "lucide-react";
import clsx from "clsx";
import { useLanguage } from "../context/LanguageContext";
import { LanguageToggle } from "./LanguageToggle";

export function Sidebar() {
  const pathname = usePathname();
  const { t } = useLanguage();

  const navItems = [
    { href: "/", label: t.masterReport, icon: ShieldAlert },
    { href: "/analyze", label: t.websiteScanner, icon: Search },
    { href: "/investigate", label: t.serverInvestigator, icon: TerminalSquare },
    { href: "/discover-apis", label: t.apiDiscoverer, icon: Network },
    { href: "/collect-services", label: t.serviceCollector, icon: Database },
    { href: "/audit-security", label: t.securityAuditor, icon: Activity }
  ];

  return (
    <aside className="fixed top-0 left-0 h-screen w-64 bg-sidebar-bg border-r border-sidebar-border z-20 flex flex-col">
      <div className="p-6">
        <Link href="/" className="flex items-center gap-4 w-full h-16 group cursor-pointer transition-all duration-500 ml-1">
          <div className="relative flex items-center justify-center w-12 h-12 rounded-xl group-hover:scale-105 transition-all duration-500 overflow-hidden">
            <motion.div
              initial={{ rotate: -10, opacity: 0, scale: 0.8 }}
              animate={{ rotate: 0, opacity: 1, scale: 1 }}
              whileHover={{ rotate: 360, scale: 1.1 }}
              transition={{ duration: 0.8, ease: "anticipate" }}
              className="relative z-10"
            >
              <Image
                src="/logo.png"
                alt="Limma Logo"
                width={36}
                height={36}
                priority
                className="object-contain drop-shadow-[0_0_12px_rgba(0,240,255,0.3)]"
              />
            </motion.div>
          </div>
          <div className="flex flex-col justify-center">
            <span className="text-[24px] font-black tracking-[0.18em] text-transparent bg-clip-text bg-gradient-to-br from-white via-gray-50 to-gray-400 drop-shadow-sm leading-none">
              LIMMA
            </span>
            <span className="text-[9px] font-mono text-cyan-400 font-bold tracking-[0.35em] uppercase opacity-90 group-hover:text-cyan-300 transition-colors mt-[5px]">
              Cybernetics
            </span>
          </div>
        </Link>
      </div>

      <nav className="flex-1 overflow-y-auto px-4 py-4 space-y-1">
        <p className="px-2 text-xs font-semibold text-gray-500 uppercase tracking-wider mb-4">{t.coreModules}</p>

        {navItems.map((item) => {
          const isActive = pathname === item.href;
          const Icon = item.icon;

          return (
            <Link
              key={item.href}
              href={item.href}
              className={clsx(
                "flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all",
                isActive
                  ? "bg-accent-cyan/10 text-accent-cyan"
                  : "text-gray-400 hover:bg-white/5 hover:text-white"
              )}
            >
              <Icon className={clsx("h-4 w-4", isActive && "text-accent-cyan")} />
              {item.label}
              {isActive && (
                <div className="ml-auto w-1.5 h-1.5 rounded-full bg-accent-cyan" />
              )}
            </Link>
          );
        })}
      </nav>

      <div className="p-4 border-t border-sidebar-border">
        <LanguageToggle />
      </div>
    </aside>
  );
}
