import React, { useState } from "react";
import Layout from "@theme/Layout";
import Link from "@docusaurus/Link";
import CodeBlock from "@theme/CodeBlock";
import useBaseUrl from "@docusaurus/useBaseUrl";
import { HtmlClassNameProvider } from "@docusaurus/theme-common";
import styles from "./index.module.css";

const INSTALL_CMD =
  "curl -fsSL https://alexpasmantier.github.io/television/install.sh | bash";

function InstallCommand() {
  const [copied, setCopied] = useState(false);

  const copy = () => {
    navigator.clipboard.writeText(INSTALL_CMD).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  };

  return (
    <div className={styles.installCmd}>
      <span className={styles.installPrompt}>$</span>
      <code>{INSTALL_CMD}</code>
      <button
        type="button"
        className={styles.copyBtn}
        onClick={copy}
        aria-label="Copy install command"
      >
        {copied ? "copied" : "copy"}
      </button>
    </div>
  );
}

const CHANNEL_SNIPPET = `[metadata]
name = "git-log"
description = "Search through git log"

[source]
command = "git log --oneline --color=always"

[preview]
command = "git show -p '{split: :0}'"`;

export default function Home() {
  return (
    <HtmlClassNameProvider className="landing-page">
    <Layout description="A very fast, portable and hackable fuzzy finder for the terminal.">
      <main className={styles.main}>
        <section className={styles.hero}>
          <div className={styles.heroInner}>
            <h1 className={styles.title}>television</h1>
            <p className={styles.tagline}>
              A very fast, portable and hackable fuzzy finder for the terminal.
            </p>
            <InstallCommand />
            <p className={styles.linkRow}>
              <Link to="/getting-started/installation">install</Link>
              <span className={styles.linkSep}>·</span>
              <Link to="/getting-started/quickstart">quickstart</Link>
              <span className={styles.linkSep}>·</span>
              <Link to="/user-guide/channels">docs</Link>
              <span className={styles.linkSep}>·</span>
              <Link href="https://github.com/alexpasmantier/television">
                github
              </Link>
            </p>
            <div className={styles.screenshotFrame}>
              <span className={styles.screenshotTitle}>tv - files</span>
              <img
                className={styles.screenshot}
                src={useBaseUrl("/img/tv-0.15.png")}
                alt="Television running in a terminal, fuzzy-searching channel files with a TOML preview panel"
              />
            </div>
          </div>
        </section>

        <section className={styles.section}>
          <div className={styles.intro}>
            <p>
              tv is a general-purpose fuzzy finder: it reads lines from a
              source command, lets you search through them in real time with a
              live preview, and prints your selection to stdout, so it
              composes with scripts and pipes like any other unix tool.
            </p>
            <CodeBlock language="bash">
              {"fd -t f | tv | xargs -o nvim"}
            </CodeBlock>
            <p>
              Searches you run often don't need to be retyped: a channel
              bundles a source command with an optional preview, keybindings,
              and actions to run on the selection when you exit, all in a
              single TOML file.
            </p>
            <CodeBlock
              language="toml"
              title="~/.config/television/cable/git-log.toml"
            >
              {CHANNEL_SNIPPET}
            </CodeBlock>
            <p>
              tv ships with channels for files, git repositories, environment
              variables, processes and more. <code>tv update-channels</code>{" "}
              pulls from the{" "}
              <Link href="https://github.com/alexpasmantier/television/tree/main/cable">
                community collection
              </Link>{" "}
              (100+ channels), or{" "}
              <Link to="/getting-started/first-channel">write your own</Link>.
            </p>
          </div>
        </section>

      </main>
    </Layout>
    </HtmlClassNameProvider>
  );
}
