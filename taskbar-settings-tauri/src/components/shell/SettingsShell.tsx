import type { ReactNode } from "react";
import SidebarNav from "../navigation/SidebarNav";

interface NavEntry {
  id: string;
  index: string;
  label: string;
}

interface SettingsShellProps {
  items: NavEntry[];
  activeId: string;
  onPageChange: (id: string) => void;
  children: ReactNode;
}

export default function SettingsShell({ items, activeId, onPageChange, children }: SettingsShellProps) {
  return (
    <div className="app">
      <SidebarNav
        activeId={activeId}
        items={items}
        onChange={onPageChange}
      />
      <div className="main">
        {children}
      </div>
    </div>
  );
}
