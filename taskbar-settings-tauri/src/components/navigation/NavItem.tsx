interface NavItemProps {
  id: string;
  index: string;
  label: string;
  active: boolean;
  onClick: (id: string) => void;
}

export default function NavItem({ id, index, label, active, onClick }: NavItemProps) {
  const className = `nav-item${active ? " nav-item--active" : ""}`;

  return (
    <button
      aria-current={active ? "page" : undefined}
      className={className}
      data-p={id}
      onClick={() => onClick(id)}
      type="button"
    >
      {index} {label}
    </button>
  );
}
