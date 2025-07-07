import React from "react";
import TabItem from "@theme/TabItem";
import { TabItemProps } from "@docusaurus/theme-common/internal";
import clsx from "clsx";
import styles from "./styles.module.css";

export default function InstallationTabItem({
  className,
  ...props
}: TabItemProps) {
  return <TabItem {...props} className={clsx(className, styles.item)} />;
}
