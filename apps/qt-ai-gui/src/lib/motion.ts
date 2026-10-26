/**
 * Cửa sổ mất focus (người dùng đang ở app khác) thì gắn data-inactive lên <html>; index.css tắt các hiệu ứng lặp vô hạn
 * (chấm "đang dịch", spinner) để webview khỏi vẽ 60 khung/giây khi không ai nhìn. Khởi đầu coi là đang dùng —
 * hasFocus() lúc boot có thể sai, chờ sự kiện focus/blur thật. Trả hàm gỡ listener.
 */
export function pauseMotionWhenInactive(win: Window, root: HTMLElement): () => void {
  const inactive = () => root.setAttribute("data-inactive", "");
  const active = () => root.removeAttribute("data-inactive");
  win.addEventListener("blur", inactive);
  win.addEventListener("focus", active);
  return () => {
    win.removeEventListener("blur", inactive);
    win.removeEventListener("focus", active);
    active();
  };
}
