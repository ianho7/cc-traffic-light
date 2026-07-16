import type { SettingsAboutMetadataDto } from "../types";
import { m } from "../paraglide/messages.js";

interface AboutPageProps {
  about: SettingsAboutMetadataDto;
}

export default function AboutPage({ about }: AboutPageProps) {
  return (
    <div className="page-body">
      <div
        className="base-card version-card"
        style={{ padding: 0, overflow: "hidden" }}
      >
        <div
          className="about-row"
          style={{
            display: "grid",
            gridTemplateColumns: "220px 1fr",
            gap: 24,
            alignItems: "center",
            padding: 34,
            borderBottom: "1px solid var(--line, #deded8)"
          }}
        >
          <div>
            <div
              className="about-label"
              style={{ fontSize: 24, fontWeight: 900 }}
            >
              {m.about_product()}
            </div>
            <div
              className="about-key"
              style={{
                font: "800 11px var(--mono)",
                letterSpacing: "0.14em",
                color: "#999",
                marginTop: 8
              }}
            >
              PRODUCT
            </div>
          </div>
          <div
            className="about-product"
            style={{
              font: "950 60px var(--ui)",
              letterSpacing: "-0.12em",
              lineHeight: 1
            }}
          >
            {about.product_name}
          </div>
        </div>

        <div
          className="about-row"
          style={{
            display: "grid",
            gridTemplateColumns: "220px 1fr",
            gap: 24,
            alignItems: "center",
            padding: 34,
            borderBottom: 0
          }}
        >
          <div>
            <div
              className="about-label"
              style={{ fontSize: 24, fontWeight: 900 }}
            >
              {m.about_version()}
            </div>
            <div
              className="about-key"
              style={{
                font: "800 11px var(--mono)",
                letterSpacing: "0.14em",
                color: "#999",
                marginTop: 8
              }}
            >
              VERSION
            </div>
          </div>
          <div
            className="version"
            style={{
              font: "950 60px var(--mono)",
              letterSpacing: "-0.12em",
              lineHeight: 1
            }}
          >
            {about.version}
          </div>
        </div>
      </div>
    </div>
  );
}
