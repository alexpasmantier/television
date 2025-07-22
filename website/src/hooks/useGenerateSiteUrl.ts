import useDocusaurusContext from "@docusaurus/useDocusaurusContext";

export const useGenerateSiteUrl = () => {
  const { siteConfig } = useDocusaurusContext();

  return (path: string) => {
    return `${siteConfig.baseUrl}${
      path.startsWith("/") ? path.slice(1) : path
    }`;
  };
};
