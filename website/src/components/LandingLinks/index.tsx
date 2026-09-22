import React from "react";
import Link from "@docusaurus/Link";
import styles from "./styles.module.css";

const LINKS = [
  { to: "/getting-started/installation", label: "Installation" },
  { to: "/getting-started/quickstart", label: "Getting Started" },
  { to: "/user-guide/channels", label: "Documentation" },
];

// navbar item type "custom-landingLinks": the landing page's menu. Centered
// in the navbar on desktop, listed in the drawer on mobile. Only shown on the
// landing page (see styles).
export default function LandingLinks({
  mobile,
  onClick,
}: {
  mobile?: boolean;
  onClick?: () => void;
}) {
  if (mobile) {
    return (
      <>
        {LINKS.map(({ to, label }) => (
          <li key={to} className={`menu__list-item ${styles.mobile}`}>
            <Link className="menu__link" to={to} onClick={onClick}>
              {label}
            </Link>
          </li>
        ))}
      </>
    );
  }

  return (
    <nav className={styles.links}>
      {LINKS.map(({ to, label }, i) => (
        <React.Fragment key={to}>
          {i > 0 && <span className={styles.sep}>·</span>}
          <Link to={to}>{label}</Link>
        </React.Fragment>
      ))}
    </nav>
  );
}
