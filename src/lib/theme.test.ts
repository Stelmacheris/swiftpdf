import { describe, expect, it } from "vitest";
import { DEFAULT_THEME_SETTING, resolveTheme } from "./theme";

describe("resolveTheme", () => {
  it("defaults to dark", () => {
    expect(resolveTheme(DEFAULT_THEME_SETTING, false)).toBe("dark");
  });

  it("keeps an explicit choice regardless of Windows", () => {
    expect(resolveTheme("dark", false)).toBe("dark");
    expect(resolveTheme("light", true)).toBe("light");
  });

  it("follows Windows when set to system", () => {
    expect(resolveTheme("system", true)).toBe("dark");
    expect(resolveTheme("system", false)).toBe("light");
  });
});
