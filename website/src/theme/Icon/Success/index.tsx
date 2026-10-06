import React, { type ReactNode } from "react";
import type { Props } from "@theme/Icon/Success";

export default function IconSuccess(props: Props): ReactNode {
  return (
    <svg viewBox="0 0 24 24" {...props}>
      <path
        d="M5 12.25 9.5 16.75 19 7.25"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
