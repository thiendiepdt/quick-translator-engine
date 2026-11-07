"""best4 = best3 + mốc tương đối, khoảng, rưỡi, đơn vị thời gian, X四周 (lấy ý từ bộ luật số thứ hai, đầu ra theo bản biên tập).

best5 = best4 bỏ hai luật thua trên dev.
"""
import pathlib

best = pathlib.Path("QT_best3.txt").read_text(encoding="utf-8")


def put_after(marker, text):
    global best
    assert marker in best, marker
    best = best.replace(marker, marker + text, 1)


put_after("[lớp]\n", "đv-thời: 天=ngày 日=ngày 年=năm 个月=tháng 个星期=tuần 星期=tuần 个礼拜=tuần 周=tuần 秒=giây 秒钟=giây 分钟=phút 小时=tiếng 个小时=tiếng 个时辰=canh_giờ 时辰=canh_giờ 世纪=thế_kỷ 个世纪=thế_kỷ 季度=quý 个季度=quý 昼夜=ngày_đêm 炷香=nén_hương\n")
put_after("[số]\n", """# Mốc tương đối và khoảng
{số:1-3}{đv-thời}(后|之后|以后) = {1} {2} sau -> thời-gian
{số:1-3}{đv-thời}(前|之前|以前) = {1} {2} trước -> thời-gian
{số:1-3}{đv-thời}(内|之内|以内) = trong vòng {1} {2} -> thời-gian
{số:1-3}{đv-thời}左右 = khoảng {1} {2} -> thời-gian
{số:1-3}(多|余){đv-thời} = hơn {1} {2} -> thời-gian
{số:1-3}{đv-thời}多 = hơn {1} {2} -> thời-gian
{số:1-3}个半小时 = {1} tiếng rưỡi -> thời-gian
{số:1-3}个半月 = {1} tháng rưỡi -> thời-gian
{số:1-3}年半 = {1} năm rưỡi -> thời-gian
{số:1-3}天半 = {1} ngày rưỡi -> thời-gian
{số:1-3}{đv-thời} = {1} {2} -> thời-gian
{số:1-2}点(到|至){số:1-2}点 = từ {1} giờ đến {2} giờ -> thời-gian
{số:1-2}月{số:1-3}(日|号)(到|至){số:1-2}月{số:1-3}(日|号) = từ ngày {2} tháng {1} đến ngày {4} tháng {3} -> thời-gian
{số:1-2}月{số:1-3}(日|号)(到|至){số:1-3}(日|号) = từ ngày {2} đến ngày {3} tháng {1} -> thời-gian
公元前{số:1-6}年 = năm {1} trước Công nguyên -> thời-gian
{buổi}{số:1-2}点多钟? = hơn {2} giờ {1} -> thời-gian
""")
# X四周 / X周围 -> xung quanh X, vào lớp vị-trí.
best = best.replace("vị-trí: ", "vị-trí: 四周=xung_quanh 周围=xung_quanh 附近=gần ", 1)
pathlib.Path("QT_best4.txt").write_text(best, encoding="utf-8")
# Dev: luật trần {số}{đv-thời} thua 45/136, 年半 chỉ hoà.
drop = {"{số:1-3}{đv-thời} = {1} {2} -> thời-gian", "{số:1-3}年半 = {1} năm rưỡi -> thời-gian"}
best5 = "\n".join(l for l in best.split("\n") if l.strip() not in drop)
pathlib.Path("QT_best5.txt").write_text(best5, encoding="utf-8")
