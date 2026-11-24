import type { ComponentProps } from "react";

type ButtonProps = ComponentProps<"button"> & { tone?: "primary" | "secondary" | "quiet" };

export function Button({
  tone = "secondary",
  className = "",
  type = "button",
  ...props
}: ButtonProps) {
  return <button type={type} className={`button button-${tone} ${className}`} {...props} />;
}

export function Badge({ className = "", ...props }: ComponentProps<"span">) {
  return <span className={`badge ${className}`} {...props} />;
}
