export type RenderMode = "sequential" | "parallel";

export interface PixelWire {
  x: number;
  y: number;
  r: number;
  g: number;
  b: number; // u8 0..255
}

export type ServerMessage =
  | { type: "FrameStart"; width: number; height: number }
  | { type: "Pixels"; pixels: PixelWire[] }
  | { type: "FrameDone" };

export interface ClientStart {
  type: "Start";
  mode: RenderMode;
  batch_size?: number;
  scene?: string;
  width?: number;
  height?: number;
}

export interface ScenesResponse {
  scenes: string[];
}

export interface ResolutionPreset {
  label: string;
  width: number;
  height: number;
}

export interface ResolutionsResponse {
  resolutions: ResolutionPreset[];
}
