// TypeScript protocol types matching Rust definitions in popup-common/src/protocol.rs

export interface PopupDefinition {
  title?: string;
  elements: Element[];
}

export type Element =
  | { type: 'text'; content: string }
  | { type: 'slider'; label: string; min: number; max: number; default?: number }
  | { type: 'checkbox'; label: string; default?: boolean }
  | { type: 'textbox'; label: string; placeholder?: string; rows?: number }
  | { type: 'multiselect'; label: string; options: string[] }
  | { type: 'choice'; label: string; options: string[]; default?: number }
  | { type: 'group'; label: string; elements: Element[] }
  | { type: 'conditional'; condition: Condition; elements: Element[] };

export type Condition =
  | string  // Simple: checkbox name
  | { field: string; value: string }  // Field check
  | { field: string; count: string }; // Count check (e.g., ">2")

export type PopupResult =
  | { status: 'completed'; button: string; [key: string]: any }
  | { status: 'cancelled' }
  | { status: 'timeout'; message: string };

// Server → Client Messages
export type ServerMessage =
  | { type: 'show_popup'; id: string; definition: PopupDefinition; timeout_ms: number }
  | { type: 'close_popup'; id: string }
  | { type: 'ping' };

// Client → Server Messages
export type ClientMessage =
  | { type: 'ready'; device_name?: string }
  | { type: 'result'; id: string; result: PopupResult }
  | { type: 'pong' };

// Helper to validate message types
export function isServerMessage(msg: any): msg is ServerMessage {
  return msg && typeof msg.type === 'string' &&
    ['show_popup', 'close_popup', 'ping'].includes(msg.type);
}

export function isClientMessage(msg: any): msg is ClientMessage {
  return msg && typeof msg.type === 'string' &&
    ['ready', 'result', 'pong'].includes(msg.type);
}
