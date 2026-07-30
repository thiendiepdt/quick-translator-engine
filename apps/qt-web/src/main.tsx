import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

// Tự host, không dùng CDN: app còn được đóng gói thành bản desktop nên phải
// chạy được offline. Đây là trục `wght` (không italic) — nhờ unicode-range,
// trình duyệt chỉ tải đúng ba lát latin + latin-ext + vietnamese.
import "@fontsource-variable/noto-sans/wght.css";
import "@fontsource-variable/noto-serif/wght.css";
import "@fontsource-variable/jetbrains-mono/wght.css";

import App from "@/app";
import { ThemeProvider } from "@/components/theme-provider";
import { Toaster } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";

import "@/index.css";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: { retry: false, refetchOnWindowFocus: false },
    mutations: { retry: false },
  },
});

const root = document.getElementById("root");
if (!root) throw new Error("Missing #root element");

createRoot(root).render(
  <StrictMode>
    <ThemeProvider>
      <QueryClientProvider client={queryClient}>
        <TooltipProvider>
          <App />
          <Toaster position="top-right" richColors />
        </TooltipProvider>
      </QueryClientProvider>
    </ThemeProvider>
  </StrictMode>,
);
