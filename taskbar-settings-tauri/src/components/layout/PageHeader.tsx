import MetaLabel from "../primitives/MetaLabel";
import SubText from "../primitives/SubText";

interface PageHeaderProps {
  meta: string;
  title: string;
  subtitle: string;
}

export default function PageHeader({ meta, title, subtitle }: PageHeaderProps) {
  return (
    <div className="page-header">
      <MetaLabel>{meta}</MetaLabel>
      <h1>{title}</h1>
      {/* <SubText>{subtitle}</SubText> */}
    </div>
  );
}
