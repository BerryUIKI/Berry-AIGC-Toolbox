import { getFileName } from "./image";

export function fileSelectionLabel(
  path: string,
  selected: boolean,
  messages: { selectFile: string; deselectFile: string },
): string {
  return (selected ? messages.deselectFile : messages.selectFile).replace("{name}", getFileName(path));
}
