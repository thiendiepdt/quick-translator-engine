"""Hai bộ chọn nghĩa, chỉ dùng nghĩa đã có trong VietPhrase của chính từ đó.

sai: nghĩa đầu sai nghĩa hẳn trong ngữ cảnh đó (vừa vặn ≠ vừa mới, ngay tại ≠ đang...).
muot: thêm các nghĩa văn phong (tựa hồ -> dường như...), giọng convert nhạt đi.
"""
import pathlib

SAI = """刚刚 = vừa -> d
刚才 = vừa rồi -> t
正在 {động-từ} = đang {1} -> v-cụm
正在 = đang -> d
正 {động-từ} = đang {1} -> v-cụm
变得 = trở nên -> v
既然 = đã -> c
谁 = ai -> r
看着 = nhìn -> v
力量 = sức mạnh -> n
身上 = trên người -> s
{động-từ} 出来 = {1} ra -> v-cụm
{người} 的 | {dấu} = của {1} -> cụm
"""
MUOT = SAI + """似乎 = dường như -> d
仿佛 = như -> d
原本 = vốn -> d
依旧 = vẫn -> d
此刻 = lúc này -> t
而且 = hơn nữa -> c
但是 = nhưng -> c
眼睛 = mắt -> n
一股 = một luồng -> m
什么 = gì -> r
怎么 = sao -> r
这 | {hệ-từ} = đây -> r
里 = trong -> f
上 = trên -> f
"""
best = pathlib.Path("QT_best3.txt").read_text(encoding="utf-8")
marker = "[nghĩa 1]\n"
assert marker in best
for name, extra in (("sai", SAI), ("muot", MUOT)):
    pathlib.Path(f"QT_nghia_{name}.txt").write_text(best.replace(marker, marker + extra, 1), encoding="utf-8")
print("ok")
