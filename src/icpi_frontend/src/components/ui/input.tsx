import * as React from "react"
import { cn } from "../../lib/utils"

export interface InputProps
  extends React.InputHTMLAttributes<HTMLInputElement> {}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, type, ...props }, ref) => {
    return (
      <input
        type={type}
        className={cn(
          "flex h-10 w-full rounded-none border border-[#1f1f1f] bg-[#141414] px-3 py-2 text-base text-white font-mono ring-offset-0 file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-[#666666] placeholder:font-sans focus-visible:outline-none focus-visible:border-[#00FF41] focus-visible:ring-1 focus-visible:ring-[#00FF41] disabled:cursor-not-allowed disabled:opacity-40 transition-colors duration-150",
          className
        )}
        ref={ref}
        {...props}
      />
    )
  }
)
Input.displayName = "Input"

export { Input }