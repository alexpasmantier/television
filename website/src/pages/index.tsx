import type { ReactNode } from "react";
import clsx from "clsx";
import Link from "@docusaurus/Link";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import Layout from "@theme/Layout";
import Heading from "@theme/Heading";
import HomepageFeatures from "@site/src/components/HomepageFeatures";

import styles from "./index.module.css";

function HomepageHeader() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header className={clsx("hero hero--primary", styles.heroBanner)}>
      <Heading as="h1" className={styles.heroTitle}>
        {siteConfig.title}
      </Heading>
      <p className={styles.heroSubtitle}>{siteConfig.tagline}</p>
    </header>
  );
}

export default function Home(): ReactNode {
  return (
    <Layout description="The cross-platform, fast and extensible fuzzy finder.">
      <div className={clsx("container", styles.main)}>
        <HomepageHeader />
        <main>
          <div className={styles.heroContent}>
            <img
              src={require("@site/static/img/tv-transparent.png").default}
              alt="Television"
              className={styles.heroImageImg}
            />
            <div className={styles.heroAbout}>
              <div>
                <p>
                  Television is a cross-platform, fast and extensible fuzzy
                  finder for the terminal.
                </p>
                <p>
                  It integrates with your shell and lets you quickly search
                  through any kind of data source (files, git repositories,
                  environment variables, docker images, you name it) using a
                  fuzzy matching algorithm and is designed to be extensible.
                </p>
                <p>
                  It is inspired by the neovim{" "}
                  <a href="https://github.com/nvim-telescope/telescope.nvim">
                    telescope
                  </a>{" "}
                  plugin and leverages{" "}
                  <a href="https://github.com/tokio-rs/tokio">tokio</a> and the
                  nucleo matcher used by{" "}
                  <a href="https://helix-editor.com/">helix</a> to ensure
                  optimal performance.
                </p>
              </div>
              <div className={styles.buttons}>
                <Link
                  className={styles.buttonGettingStarted}
                  to="/docs/Users/installation"
                >
                  Getting Started
                </Link>
              </div>
            </div>
          </div>
          <HomepageFeatures />
        </main>
      </div>
    </Layout>
  );
}
