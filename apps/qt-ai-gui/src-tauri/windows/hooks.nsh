; Hook cho installer NSIS của Tauri (bundle.windows.nsis.installerHooks), chạy cuối Section Install.
;
; Template gốc bỏ qua bước tạo shortcut khi chạy ở chế độ /UPDATE (updater trong app gọi
; `setup.exe /P /R /UPDATE`), vì cho rằng lần cài đầu đã tạo. Máy chưa từng cài tử tế — chạy bản portable,
; chạy exe rời trong target/, hay bản cũ mang tên khác — update xong không có shortcut nào, Windows search
; không thấy app. Chưa có shortcut Start Menu là dấu hiệu chưa từng cài tử tế → tạo cả Start Menu lẫn
; Desktop như một lần cài mới (ô "Create desktop shortcut" mặc định tick). Đã cài tử tế rồi tự xoá Desktop
; thì Start Menu vẫn còn nên không bị tạo lại. Tôn trọng cờ /NS (không tạo shortcut).
!macro NSIS_HOOK_POSTINSTALL
  ${If} $NoShortcutMode <> 1
  ${AndIfNot} ${FileExists} "$SMPROGRAMS\${PRODUCTNAME}.lnk"
    CreateShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    !insertmacro SetLnkAppUserModelId "$SMPROGRAMS\${PRODUCTNAME}.lnk"
    ${IfNot} ${FileExists} "$DESKTOP\${PRODUCTNAME}.lnk"
      CreateShortcut "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
      !insertmacro SetLnkAppUserModelId "$DESKTOP\${PRODUCTNAME}.lnk"
    ${EndIf}
  ${EndIf}
!macroend
