import { describe, expect, it } from "vitest";
import { normalizeSelectorList } from "./manifest";

describe("normalizeSelectorList", () => {
  it("deduplicates empty whitespace entries by filtering them out", () => {
    expect(normalizeSelectorList(" idle01, , tap_body ,  ")).toEqual([
      "idle01",
      "tap_body",
    ]);
  });
});
