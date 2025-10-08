import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"
import { cn } from "@/lib/utils"

const alertVariants = cva(
  "relative w-full p-3 border-l-4 border-y border-r flex items-start gap-3 [&>svg]:w-5 [&>svg]:h-5 [&>svg]:flex-shrink-0 [&>svg]:mt-0.5",
  {
    variants: {
      variant: {
        default: "border-l-[#00FF41] border-y-[#00FF4160] border-r-[#00FF4160] bg-[#00FF4120] [&>svg]:text-[#00FF41]",
        destructive:
          "border-l-[#FF0055] border-y-[#FF005540] border-r-[#FF005540] bg-[#FF005520] [&>svg]:text-[#FF0055]",
        success:
          "border-l-[#00FF41] border-y-[#00FF4160] border-r-[#00FF4160] bg-[#00FF4120] [&>svg]:text-[#00FF41]",
        warning:
          "border-l-[#FFE600] border-y-[#FFE60040] border-r-[#FFE60040] bg-[#FFE60020] [&>svg]:text-[#FFE600]",
        info:
          "border-l-[#00D9FF] border-y-[#00D9FF40] border-r-[#00D9FF40] bg-[#00D9FF20] [&>svg]:text-[#00D9FF]",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
)

const Alert = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & VariantProps<typeof alertVariants>
>(({ className, variant, ...props }, ref) => (
  <div
    ref={ref}
    role="alert"
    className={cn(alertVariants({ variant }), className)}
    {...props}
  />
))
Alert.displayName = "Alert"

const AlertTitle = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLHeadingElement>
>(({ className, ...props }, ref) => (
  <h5
    ref={ref}
    className={cn("text-white text-sm font-sans font-semibold", className)}
    {...props}
  />
))
AlertTitle.displayName = "AlertTitle"

const AlertDescription = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLParagraphElement>
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    className={cn("text-[#999999] text-xs font-sans leading-relaxed", className)}
    {...props}
  />
))
AlertDescription.displayName = "AlertDescription"

export { Alert, AlertTitle, AlertDescription }