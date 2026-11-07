"""Bộ mặc định = best5 + sửa nghĩa sai (đã lọc). Nhóm giọng-mượt là file riêng, tắt mặc định."""
import pathlib

SAI = """刚刚 = vừa -> d
正 {động-từ} = đang {1} -> v-cụm
正在 = đang -> d
变得 = trở nên -> v
既然 = đã -> c
谁 = ai -> r
看着 = nhìn -> v
力量 = sức mạnh -> n
身上 = trên người -> s
{động-từ} 出来 = {1} ra -> v-cụm
"""
# Dev: >= 70%, bỏ 什么=gì (30%), 但是 (51%), 上/里 trần (43%/31%), 这|是 (63%), 怎么 (68%).
MUOT = """[giọng-mượt 1]
似乎 = dường như -> d
仿佛 = như -> d
原本 = vốn -> d
依旧 = vẫn -> d
此刻 = lúc này -> t
而且 = hơn nữa -> c
眼睛 = mắt -> n
一股 = một luồng -> m
"""
best = pathlib.Path("QT_best5.txt").read_text(encoding="utf-8")
marker = "[nghĩa 1]\n"
assert marker in best
default = best.replace(marker, marker + SAI, 1)
pathlib.Path("QT_final.txt").write_text(default, encoding="utf-8")
pathlib.Path("QT_final_muot.txt").write_text(default.rstrip("\n") + "\n\n" + MUOT, encoding="utf-8")
pathlib.Path("QuyTac-giong-muot.txt").write_text(MUOT, encoding="utf-8")
print("mặc định", default.count(" = "), "luật; giọng-mượt", MUOT.count(" = "), "luật")
