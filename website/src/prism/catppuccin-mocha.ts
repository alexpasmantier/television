import type { PrismTheme } from "prism-react-renderer";

// https://catppuccin.com/palette, following the catppuccin style guide for
// syntax highlighting. The code block background is set in custom.css.
const catppuccinMocha: PrismTheme = {
  plain: { color: "#cdd6f4", backgroundColor: "#1e1e2e" },
  styles: [
    {
      types: ["comment", "prolog", "doctype", "cdata", "shebang"],
      style: { color: "#9399b2", fontStyle: "italic" },
    },
    { types: ["punctuation"], style: { color: "#9399b2" } },
    {
      types: ["keyword", "atrule", "important", "for-or-select"],
      style: { color: "#cba6f7" },
    },
    {
      types: ["string", "char", "attr-value", "inserted"],
      style: { color: "#a6e3a1" },
    },
    {
      types: ["number", "boolean", "constant", "builtin", "symbol", "date", "environment"],
      style: { color: "#fab387" },
    },
    {
      types: ["function", "function-name", "tag", "selector", "property", "key"],
      style: { color: "#89b4fa" },
    },
    {
      types: ["class-name", "table", "attr-name", "maybe-class-name"],
      style: { color: "#f9e2af" },
    },
    { types: ["operator", "entity", "url", "coord"], style: { color: "#89dceb" } },
    {
      types: ["variable", "parameter", "assign-left"],
      style: { color: "#cdd6f4" },
    },
    { types: ["regex", "escape"], style: { color: "#f5c2e7" } },
    { types: ["deleted"], style: { color: "#f38ba8" } },
    { types: ["changed"], style: { color: "#f9e2af" } },
  ],
};

export default catppuccinMocha;
