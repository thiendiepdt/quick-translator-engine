import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { ExtIcon } from "@/components/ext-icon";

describe("ExtIcon", () => {
  it("có icon thì hiện ảnh; không có thì chữ cái đầu của tên", () => {
    const { container, rerender } = render(<ExtIcon icon="data:image/png;base64,AAAA" name="Fanqie 番茄" />);
    expect(container.querySelector("img")).toHaveAttribute("src", "data:image/png;base64,AAAA");

    rerender(<ExtIcon icon={null} name="fanqie 番茄" />);
    expect(container.querySelector("img")).toBeNull();
    expect(container).toHaveTextContent("F");

    rerender(<ExtIcon name="番茄小说" />);
    expect(container).toHaveTextContent("番");
  });
});
