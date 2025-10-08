import * as React from "react"
import { Slot } from "@radix-ui/react-slot"
import { cva, type VariantProps } from "class-variance-authority"
import { cn } from "../../lib/utils"

const buttonVariants = cva(
  "inline-flex items-center justify-center whitespace-nowrap font-sans text-sm font-medium transition-colors duration-150 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-[#00FF41] focus-visible:ring-offset-0 disabled:pointer-events-none disabled:opacity-40 active:scale-[0.98]",
  {
    variants: {
      variant: {
        default: "rounded-sm border border-[#00FF4160] bg-[#0a0a0a] text-[#00FF41] hover:bg-[#00FF4120] hover:border-[#00FF41] active:bg-[#00FF41] active:text-[#000000] active:brightness-90",
        primary: "rounded-sm bg-[#00FF41] text-[#000000] border border-[#00FF41] hover:bg-[#00CC34] active:bg-[#00AA2A] active:brightness-90",
        destructive: "rounded-sm bg-[#FF0055] text-white border border-[#FF0055] hover:bg-[#CC0044] active:bg-[#AA0037] active:brightness-90",
        outline: "rounded-sm border border-[#00FF4160] bg-[#0a0a0a] text-[#00FF41] hover:bg-[#00FF4120] hover:border-[#00FF41] active:bg-[#00FF4140] active:brightness-90",
        secondary: "rounded-sm border border-[#1f1f1f] bg-[#0a0a0a] text-[#999999] hover:bg-[#1f1f1f] active:bg-[#2a2a2a] active:brightness-90",
        ghost: "bg-transparent border border-[#1f1f1f] text-[#999999] hover:border-[#00FF4160] hover:text-[#00FF41] active:bg-[#00FF4120] active:brightness-90",
        link: "text-[#00D9FF] underline-offset-4 hover:underline active:brightness-75",
      },
      size: {
        default: "px-3 py-2",
        sm: "px-2 py-1 text-xs",
        lg: "px-4 py-2.5 text-base",
        icon: "h-10 w-10",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
)

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : "button"
    return (
      <Comp
        className={cn(buttonVariants({ variant, size, className }))}
        ref={ref}
        {...props}
      />
    )
  }
)
Button.displayName = "Button"

export { Button, buttonVariants }