import type { ReactNode } from "react";
import clsx from "clsx";
import Heading from "@theme/Heading";
import styles from "./styles.module.css";
import { useGenerateSiteUrl } from "@site/src/hooks";

type FeatureItem = {
  title: string;
  imgSrc: string;
  description: ReactNode;
  link?: string;
};

const FeatureList: FeatureItem[] = [
  {
    title: "Create your own channels",
    imgSrc: require("@site/static/img/files-toml.png").default,
    description: (
      <>
        Create your own channels in a simple TOML file and search through files,
        git repositories, environment variables, docker images, and more.
      </>
    ),
    link: "/docs/Users/channels",
  },
  {
    title: "Integrates with your shell",
    imgSrc: require("@site/static/img/zsh-integration.png").default,
    description: (
      <>
        Television integrates with your shell and provides autocompletion that
        is both extensible and configurable to use your own channels.
      </>
    ),
    link: "/docs/Users/shell-integration",
  },
];

function Feature({ title, imgSrc, description, link }: FeatureItem) {
  const generateSiteUrl = useGenerateSiteUrl();

  return (
    <div className={clsx("col col--6", styles.featureItem)}>
      <div
        className={clsx(
          "text--center padding-horiz--md",
          styles.titleContainer
        )}
      >
        <Heading as="h3">
          <a href={generateSiteUrl(link)}>{title}</a>
        </Heading>
        <p>{description}</p>
      </div>
      <div className="featureImageContainer">
        <img
          src={imgSrc}
          alt={title}
          className={styles.featureImage}
          role="img"
        />
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className={clsx("row", styles.featuresRow)}>
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
