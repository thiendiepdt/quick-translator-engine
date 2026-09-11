# Vòng lặp dịch tự động qua Antigravity CLI (agy): mỗi phiên agy -p là một
# context sạch, thay cho việc tự tay mở conversation mới trong IDE.
# Trong lúc agy chạy, script poll state.json để in tiến độ từng chương.
# Dùng: .\auto-translate.ps1 -Root D:\stories\ten-truyen [-MaxSessions 50]
param(
  [Parameter(Mandatory)][string]$Root,
  [int]$MaxSessions = 50,
  [int]$PollSeconds = 5
)
$ErrorActionPreference = "Stop"
$CliDir = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$Root = (Resolve-Path $Root).Path
$StateJson = Join-Path $Root "state.json"

# Đọc thẳng state.json (ghi atomic nên không bao giờ đọc trúng file dở);
# nhanh hơn nhiều so với gọi qt-ai status qua npm mỗi lần poll.
function Get-Counts {
  $state = Get-Content $StateJson -Raw | ConvertFrom-Json
  $c = @{ queued = 0; translating = 0; done = 0; error = 0; skipped = 0 }
  $current = ""
  foreach ($prop in $state.chapters.PSObject.Properties) {
    $c[$prop.Value.status] += 1
    if ($prop.Value.status -eq "translating") { $current = $prop.Name }
  }
  [pscustomobject]@{
    Done = $c.done; Queued = $c.queued; Translating = $c.translating
    Current = $current
    Settled = $c.done + $c.error + $c.skipped   # chương đã xử lý xong, không quay lại
  }
}

function Format-Counts($c) {
  $line = "done $($c.Done), queued $($c.Queued)"
  if ($c.Current) { $line += ", đang dịch chương $($c.Current)" }
  $line
}

# Đường dẫn tuyệt đối để agent mở thẳng file, không đi search filesystem
$workflowPath = Join-Path $Root ".agent\workflows\translate.md"
$prompt = "Mở file $workflowPath (đường dẫn tuyệt đối, tồn tại sẵn, KHÔNG cần tìm kiếm) rồi làm đúng theo nó: dịch tới khi chạm giới hạn chương/phiên hoặc hết hàng đợi thì dừng. Thư mục truyện: $Root"

Push-Location $Root
try {
  for ($i = 1; $i -le $MaxSessions; $i++) {
    $before = Get-Counts
    if ($before.Queued -eq 0 -and $before.Translating -eq 0) {
      Write-Host "Hết hàng đợi — done $($before.Done)."; break
    }
    Write-Host "=== Phiên $i — $(Format-Counts $before) ==="

    # agy chạy nền (cwd = folder truyện để tự nhặt AGENTS.md), script poll tiến độ
    $proc = Start-Process agy -ArgumentList @("-p", "`"$prompt`"", "--dangerously-skip-permissions") `
      -WorkingDirectory $Root -NoNewWindow -PassThru
    $last = Format-Counts $before
    while (-not $proc.HasExited) {
      Start-Sleep -Seconds $PollSeconds
      try { $now = Format-Counts (Get-Counts) } catch { continue }
      if ($now -ne $last) {
        Write-Host ("[{0:HH:mm:ss}] {1}" -f (Get-Date), $now)
        $last = $now
      }
    }

    $after = Get-Counts
    if ($after.Settled -le $before.Settled) {
      Write-Warning "Phiên $i không chốt thêm chương nào (kẹt/refuse/hết quota?) — dừng để khỏi đốt quota. Xem work/ và chạy status."
      break
    }
  }
} finally {
  Pop-Location
  npm --prefix $CliDir run -s qt-ai -- status $Root
}
