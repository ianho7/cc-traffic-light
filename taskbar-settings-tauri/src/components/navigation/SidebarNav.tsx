import BrandMark from "./BrandMark";
import NavItem from "./NavItem";
import { m } from "../../paraglide/messages.js";

interface NavEntry {
  id: string;
  index: string;
  label: string;
}

interface SidebarNavProps {
  items: NavEntry[];
  activeId: string;
  onChange: (id: string) => void;
}

export default function SidebarNav({ items, activeId, onChange }: SidebarNavProps) {
  return (
    <aside className="side">
      <BrandMark />
      <nav aria-label={m.shell_nav_aria_label()} className="nav">
        {items.map((entry) => (
          <NavItem
            active={entry.id === activeId}
            id={entry.id}
            index={entry.index}
            key={entry.id}
            label={entry.label}
            onClick={onChange}
          />
        ))}
      </nav>
    </aside>
  );
}
