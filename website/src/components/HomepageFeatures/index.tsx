import type { ReactNode } from "react";
import clsx from "clsx";
import Heading from "@theme/Heading";
import styles from "./styles.module.css";

type FeatureItem = {
  title: string;
  Svg: React.ComponentType<React.ComponentProps<"svg">>;
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: "Integrates with your shell",
    Svg: require("@site/static/img/integrate_icon.svg").default,
    description: (
      <>
        Television integrates with your shell and lets you quickly search
        through any kind of data source (files, git repositories, environment
        variables, docker images, you name it) using a fuzzy matching algorithm
        and is designed to be extensible.
      </>
    ),
  },
  {
    title: "Insipred by great tools",
    Svg: require("@site/static/img/inspired_icon.svg").default,
    description: (
      <>
        It is inspired by the neovim{" "}
        <a href="https://github.com/nvim-telescope/telescope.nvim">telescope</a>{" "}
        plugin and leverages{" "}
        <a href="https://github.com/tokio-rs/tokio">tokio</a> and the{" "}
        <a href="https://github.com/helix-editor/nucleo">nucleo</a> matcher used
        by the <a href="https://github.com/helix-editor/helix">helix</a> editor
        to ensure optimal performance.
      </>
    ),
  },
];

function Feature({ title, Svg, description }: FeatureItem) {
  return (
    <div className={clsx("col col--6", styles.featureItem)}>
      <div className="text--center">
        <Svg className={styles.featureSvg} role="img" />
      </div>
      <div
        className={clsx(
          "text--center padding-horiz--md",
          styles.titleContainer
        )}
      >
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
