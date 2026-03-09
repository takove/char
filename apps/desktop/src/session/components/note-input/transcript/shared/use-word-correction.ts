import { generateText } from "ai";
import { useCallback, useState } from "react";

import type { SegmentWord } from "@hypr/transcript";

import { useLanguageModel } from "~/ai/hooks";

export type CorrectionState = {
  isLoading: boolean;
  suggestions: string[];
  error?: string;
  targetWord?: SegmentWord;
};

export function useWordCorrection(getContextWords: () => SegmentWord[]) {
  const model = useLanguageModel("enhance");
  const [state, setState] = useState<CorrectionState>({
    isLoading: false,
    suggestions: [],
  });

  const suggestCorrection = useCallback(
    async (word: SegmentWord) => {
      if (!model) {
        setState({ isLoading: false, suggestions: [], error: "No AI model configured" });
        return;
      }

      setState({ isLoading: true, suggestions: [], targetWord: word });

      const contextWords = getContextWords();
      const wordIndex = contextWords.findIndex((w) => w.id === word.id);
      const start = Math.max(0, wordIndex - 10);
      const end = Math.min(contextWords.length, wordIndex + 11);
      const context = contextWords
        .slice(start, end)
        .map((w) => w.text.trim())
        .join(" ");

      const prompt = `Given this transcript context: "${context}"

The word "${word.text.trim()}" (confidence: ${Math.round((word.confidence ?? 0) * 100)}%) may be incorrectly transcribed.

Suggest up to 3 likely correct alternatives for this word. Consider the surrounding context. Return ONLY the suggested words, one per line, nothing else.`;

      try {
        const result = await generateText({
          model,
          prompt,
          maxTokens: 50,
        });

        const suggestions = result.text
          .split("\n")
          .map((s) => s.trim())
          .filter((s) => s.length > 0 && s.length < 50)
          .slice(0, 3);

        setState({ isLoading: false, suggestions, targetWord: word });
      } catch {
        setState({
          isLoading: false,
          suggestions: [],
          error: "Failed to generate suggestions",
          targetWord: word,
        });
      }
    },
    [model, getContextWords],
  );

  const reset = useCallback(() => {
    setState({ isLoading: false, suggestions: [] });
  }, []);

  return { state, suggestCorrection, reset, hasModel: !!model };
}
