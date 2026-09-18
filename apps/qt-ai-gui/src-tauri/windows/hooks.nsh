; Hook cho installer NSIS của Tauri (bundle.windows.nsis.installerHooks), chạy cuối Section Install.
;
; Template gốc bỏ qua bước tạo shortcut khi chạy ở chế độ /UPDATE (updater trong app gọi
; `setup.exe /P /R /UPDATE`), vì cho rằng lần cài đầu đã tạo. Máy chưa từng cài tử tế — chạy bản portable,
; chạy exe rời trong target/, hay bản cũ mang tên khác — update xong không có shortcut nào, Windows search
; không thấy app. Tạo bù shortcut Start Menu nếu chưa có; tôn trọng cờ /NS (không tạo shortcut).
; Desktop không đụng: đó là lựa chọn của người dùng lúc cài.
!macro NSIS_HOOK_POSTINSTALL
  ${If} $NoShortcutMode <> 1
  ${AndIfNot} ${FileExists} "$SMPROGRAMS\${PRODUCTNAME}.lnk"
    CreateShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    !insertmacro SetLnkAppUserModelId "$SMPROGRAMS\${PRODUCTNAME}.lnk"
  ${EndIf}
!macroend
