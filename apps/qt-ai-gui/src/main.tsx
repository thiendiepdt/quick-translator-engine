import "@fontsource-variable/inter";
import "@fontsource-variable/jetbrains-mono";
import "@fontsource-variable/noto-serif";

import { ThemeProvider } from "next-themes";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import App from "@/app";
import { Toaster } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { bootPalette, THEME_STORAGE_KEY } from "@/lib/theme";

import "@/index.css";

// Áp bộ màu cache trước khi render để không nháy; AppConfig ghi đè khi nạp (hooks/use-theme).
bootPalette(document.documentElement, localStorage);

const root = document.getElementById("root");
if (!root) throw new Error("Thiếu #root");

createRoot(root).render(
  <StrictMode>
    <ThemeProvider
      attribute="class"
      defaultTheme="system"
      enableSystem
      storageKey={THEME_STORAGE_KEY}
      disableTransitionOnChange
    >
      <TooltipProvider>
        <App />
        <Toaster position="top-right" richColors />
      </TooltipProvider>
    </ThemeProvider>
  </StrictMode>,
);
