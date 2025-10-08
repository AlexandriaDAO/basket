import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"
import { cn } from "@/lib/utils"

const badgeVariants = cva(
  "inline-flex items-center rounded-sm border border-current px-2 py-0.5 font-mono text-xs uppercase tracking-wider transition-none focus:outline-none",
  {
    variants: {
      variant: {
        default:
          "bg-transparent border-[#1f1f1f] text-[#999999]",
        secondary:
          "bg-transparent border-[#1f1f1f] text-[#999999]",
        destructive:
          "bg-[#FF005520] border-[#FF0055] text-[#FF0055]",
        outline: "bg-transparent border-[#1f1f1f] text-[#999999]",
        success:
          "bg-[#00FF4120] border-[#00FF41] text-[#00FF41]",
        warning:
          "bg-[#FFE60020] border-[#FFE600] text-[#FFE600]",
        error:
          "bg-[#FF005520] border-[#FF0055] text-[#FF0055]",
        info:
          "bg-[#00D9FF20] border-[#00D9FF] text-[#00D9FF]",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
)

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, ...props }: BadgeProps) {
  return (
    <div className={cn(badgeVariants({ variant }), className)} {...props} />
  )
}

export { Badge, badgeVariants }