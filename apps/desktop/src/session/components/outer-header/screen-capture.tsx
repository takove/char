import { invoke } from "@tauri-apps/api/core";
import { tempDir } from "@tauri-apps/api/path";
import { CameraIcon, CircleIcon, SquareIcon } from "lucide-react";
import { useCallback, useState } from "react";

import { commands as fsSyncCommands } from "@hypr/plugin-fs-sync";
import { Button } from "@hypr/ui/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@hypr/ui/components/ui/tooltip";
import { cn } from "@hypr/utils";

export function ScreenCaptureButtons({
  sessionId,
}: {
  sessionId: string;
}) {
  const [isRecording, setIsRecording] = useState(false);

  const handleScreenshot = useCallback(async () => {
    try {
      const tmpDir = await tempDir();
      const result = await invoke<{ path: string; filename: string }>(
        "plugin:screen|capture_screenshot",
        { outputDir: tmpDir },
      );

      const response = await fetch(`file://${result.path}`);
      const arrayBuffer = await response.arrayBuffer();
      const data = Array.from(new Uint8Array(arrayBuffer));

      await fsSyncCommands.attachmentSave(sessionId, data, result.filename);
    } catch (error) {
      console.error("[screen-capture] screenshot failed:", error);
    }
  }, [sessionId]);

  const handleToggleRecording = useCallback(async () => {
    try {
      if (isRecording) {
        const result = await invoke<{ path: string; filename: string } | null>(
          "plugin:screen|stop_recording",
        );
        setIsRecording(false);

        if (result) {
          const response = await fetch(`file://${result.path}`);
          const arrayBuffer = await response.arrayBuffer();
          const data = Array.from(new Uint8Array(arrayBuffer));
          await fsSyncCommands.attachmentSave(sessionId, data, result.filename);
        }
      } else {
        const tmpDir = await tempDir();
        await invoke("plugin:screen|start_recording", {
          outputDir: tmpDir,
          sessionId,
        });
        setIsRecording(true);
      }
    } catch (error) {
      console.error("[screen-capture] recording toggle failed:", error);
      setIsRecording(false);
    }
  }, [sessionId, isRecording]);

  return (
    <div className="flex items-center gap-0.5">
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            className="size-7"
            onClick={handleScreenshot}
          >
            <CameraIcon className="size-3.5" />
          </Button>
        </TooltipTrigger>
        <TooltipContent>Take screenshot</TooltipContent>
      </Tooltip>

      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            className={cn([
              "size-7",
              isRecording && "text-red-500 hover:text-red-600",
            ])}
            onClick={handleToggleRecording}
          >
            {isRecording
              ? <SquareIcon className="size-3 fill-current" />
              : <CircleIcon className="size-3.5" />}
          </Button>
        </TooltipTrigger>
        <TooltipContent>
          {isRecording ? "Stop recording" : "Record screen"}
        </TooltipContent>
      </Tooltip>
    </div>
  );
}
