import {
  LayoutDashboard,
  Radar,
  ShieldCheck,
  Zap,
  FlaskConical,
  FileText,
  Settings,
  Search,
  BarChart3,
  History,
  Shield,
  Eye,
  Globe,
  type LucideIcon,
} from "lucide-react";
import { ROUTES } from "./routes";

export interface NavItem {
  label: string;
  href: string;
  icon: LucideIcon;
  badge?: string;
}

export interface NavGroup {
  label: string;
  items: NavItem[];
}

export const NAVIGATION: NavGroup[] = [
  {
    label: "Overview",
    items: [
      { label: "Dashboard", href: ROUTES.DASHBOARD, icon: LayoutDashboard },
    ],
  },
  {
    label: "Reconnaissance",
    items: [
      { label: "Discovery", href: ROUTES.DISCOVERY, icon: Search },
      { label: "Analysis", href: ROUTES.ANALYSIS, icon: BarChart3 },
    ],
  },
  {
    label: "Scanning",
    items: [
      { label: "Scanner", href: ROUTES.SCANNER, icon: Radar },
      { label: "Audit", href: ROUTES.AUDIT, icon: ShieldCheck },
      { label: "Active Detection", href: ROUTES.ACTIVE_DETECTION, icon: Zap },
      { label: "Blind Scanner", href: ROUTES.BLIND_SCANNER, icon: Eye },
    ],
  },
  {
    label: "Exploitation",
    items: [
      { label: "PoC Lab", href: ROUTES.POC_LAB, icon: FlaskConical },
    ],
  },
  {
    label: "Intelligence",
    items: [
      { label: "Reports", href: ROUTES.REPORTS, icon: FileText },
      { label: "Scan History", href: ROUTES.HISTORY, icon: History },
    ],
  },

  {
    label: "System",
    items: [
      { label: "HTTP Requester", href: ROUTES.HTTP_REQUESTER, icon: Globe },
      { label: "Settings", href: ROUTES.SETTINGS, icon: Settings },
      { label: "Rule Engine", href: ROUTES.RULES, icon: Shield },
    ],
  },
];
