import React from "react";
import Tabs from "@theme/Tabs";
import { TabsProps } from "@docusaurus/theme-common/internal";
import styles from "./styles.module.css";
import clsx from "clsx";

export default function InstallationTabs({ className, ...props }: TabsProps) {
  return <Tabs {...props} className={clsx(styles.container, className)} />;
}
