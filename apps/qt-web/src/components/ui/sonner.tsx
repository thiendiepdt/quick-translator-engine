import {
  CircleCheckIcon,
  InfoIcon,
  Loader2Icon,
  OctagonXIcon,
  TriangleAlertIcon,
  XIcon,
} from "lucide-react"
import { Toaster as Sonner, type ToasterProps } from "sonner"

import { useTheme } from "@/lib/theme-context"

const Toaster = ({ ...props }: ToasterProps) => {
  const { scheme } = useTheme()

  return (
    <Sonner
      theme={scheme}
      // pointer-events-auto: dialog Radix (modal) đặt body về pointer-events
      // none; sonner không tự bật lại nên click sẽ XUYÊN QUA toast, rơi trúng
      // overlay và bị tính là "bấm ra ngoài" làm đóng dialog. Bật lại ở đây,
      // phần chặn đóng-dialog nằm trong DialogContent (isToastInteraction).
      className="toaster group pointer-events-auto"
      icons={{
        success: <CircleCheckIcon className="size-4" />,
        info: <InfoIcon className="size-4" />,
        warning: <TriangleAlertIcon className="size-4" />,
        error: <OctagonXIcon className="size-4" />,
        loading: <Loader2Icon className="size-4 animate-spin" />,
        close: <XIcon className="size-5" />,
      }}
      style={
        {
          "--normal-bg": "var(--popover)",
          "--normal-text": "var(--popover-foreground)",
          "--normal-border": "var(--border)",
          "--border-radius": "var(--radius)",
        } as React.CSSProperties
      }
      {...props}
    />
  )
}

export { Toaster }
