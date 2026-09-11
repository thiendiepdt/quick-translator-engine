/**
 * Chép text vào clipboard. navigator.clipboard cần secure context + cử chỉ người dùng — trong Tauri
 * (tauri://localhost, hay http://localhost lúc dev) đủ điều kiện; WebKitGTK cũ hoặc bị chặn thì rơi về
 * execCommand("copy") qua textarea tạm.
 */
export async function copyText(text: string): Promise<void> {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return;
    }
  } catch {
    // rơi xuống execCommand
  }
  const box = document.createElement("textarea");
  box.value = text;
  box.setAttribute("readonly", "");
  box.style.position = "fixed";
  box.style.opacity = "0";
  document.body.appendChild(box);
  box.select();
  const ok = document.execCommand("copy");
  box.remove();
  if (!ok) throw new Error("Không chép được vào clipboard");
}
