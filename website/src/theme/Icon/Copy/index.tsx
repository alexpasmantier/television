import React, { type ReactNode } from "react";
import type { Props } from "@theme/Icon/Copy";

export default function IconCopy(props: Props): ReactNode {
  return (
    <svg viewBox="0 0 24 24" {...props}>
      <g
        fill="none"
        stroke="currentColor"
        strokeWidth="1.75"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <rect x="8" y="8" width="13" height="13" rx="2.5" />
        <path d="M16 8V5.5A2.5 2.5 0 0 0 13.5 3h-8A2.5 2.5 0 0 0 3 5.5v8A2.5 2.5 0 0 0 5.5 16H8" />
      </g>
    </svg>
  );
}
