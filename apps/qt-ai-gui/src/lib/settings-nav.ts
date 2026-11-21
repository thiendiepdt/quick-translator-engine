export type SettingsSection = { id: string; label: string };

/** Khối đã cuộn qua mép trên khung trong khoảng này vẫn tính là đang xem (bù `scroll-mt` của Card). */
const TOP_SLACK = 24;

/**
 * Khối đang xem: khối cuối cùng có mép trên đã lên tới đầu khung (`offsets` = top khối − top khung). Chưa khối nào
 * lên tới thì khối đầu; cuộn hết trang thì khối cuối, vì khối cuối ngắn không bao giờ lên được đầu khung.
 */
export function pickActiveSection(offsets: number[], atBottom: boolean, threshold = TOP_SLACK): number {
  if (atBottom && offsets.length > 0) return offsets.length - 1;
  let active = 0;
  offsets.forEach((offset, index) => {
    if (offset <= threshold) active = index;
  });
  return active;
}
