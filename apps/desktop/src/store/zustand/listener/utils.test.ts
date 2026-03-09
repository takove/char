import { describe, expect, test } from "vitest";

import { fixSpacingForWords, transformWordEntries } from "./utils";

describe("fixSpacingForWords", () => {
  const testCases = [
    {
      transcript: "Hello",
      input: ["Hello"],
      output: [" Hello"],
    },
    {
      transcript: "Yes. Because we",
      input: ["Yes.", "Because", "we"],
      output: [" Yes.", " Because", " we"],
    },
    {
      transcript: "shouldn't",
      input: ["shouldn", "'t"],
      output: [" shouldn", "'t"],
    },
    {
      transcript: "Yes. Because we shouldn't be false.",
      input: ["Yes.", "Because", "we", "shouldn", "'t", "be", "false."],
      output: [" Yes.", " Because", " we", " shouldn", "'t", " be", " false."],
    },
  ];

  test.each(testCases)(
    "transcript: $transcript",
    ({ transcript, input, output }) => {
      expect(output.join("")).toEqual(` ${transcript}`);

      const actual = fixSpacingForWords(input, transcript);
      expect(actual).toEqual(output);
    },
  );
});

describe("transformWordEntries", () => {
  test("preserves confidence from word entries", () => {
    const entries = [
      { word: "hello", start: 0, end: 0.5, confidence: 0.95 },
      { word: "world", start: 0.5, end: 1.0, confidence: 0.42 },
    ];

    const [words] = transformWordEntries(entries, "hello world", 0);

    expect(words[0].confidence).toBe(0.95);
    expect(words[1].confidence).toBe(0.42);
  });

  test("confidence is undefined when not provided", () => {
    const entries = [{ word: "hello", start: 0, end: 0.5 }];

    const [words] = transformWordEntries(entries, "hello", 0);

    expect(words[0].confidence).toBeUndefined();
  });
});
