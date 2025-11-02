// TypeScript protocol types matching Rust definitions in popup-common/src/protocol.rs

export interface PopupDefinition {
  title?: string;
  elements: Element[];
}

/**
 * V2 Element format using element-as-key pattern
 * All interactive elements require 'id' field for state tracking
 */
export type Element =
  | { text: string; id?: string; when?: string }
  | { slider: string; id: string; min: number; max: number; default?: number; when?: string; reveals?: Element[] }
  | { checkbox: string; id: string; default?: boolean; when?: string; reveals?: Element[] }
  | { textbox: string; id: string; placeholder?: string; rows?: number; when?: string }
  | { multiselect: string; id: string; options: string[]; when?: string; reveals?: Element[]; [optionKey: string]: any }
  | { choice: string; id: string; options: string[]; default?: number; when?: string; reveals?: Element[]; [optionKey: string]: any }
  | { group: string; elements: Element[]; when?: string }

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
