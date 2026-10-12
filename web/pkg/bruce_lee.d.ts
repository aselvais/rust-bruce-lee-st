/* tslint:disable */
/* eslint-disable */

export class WebGame {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * 0 title, 1 level, 2 clear fanfare, 3 clear bed, 4 game over, then 5..=14 effects.
     */
    clip_samples(id: number): Float32Array;
    height(): number;
    constructor();
    pixel_len(): number;
    /**
     * Byte offset of the RGBA framebuffer in wasm memory. Stable for the life of this game.
     */
    pixels(): number;
    sample_rate(): number;
    take_sfx(): Uint8Array;
    /**
     * Advance one frame.
     *
     * `bits` uses the same edges as the desktop window:
     * 0 left, 1 right, 2 up, 3 down, 4 jump edge (Space), 5 jump held,
     * 6 up edge, 7 attack edge, 8 start edge (Enter), 9 pause edge,
     * 10 debug outline (F1).
     * `dt` is seconds and is clamped to the desktop range.
     * The return value is the mode: 0 title, 1 play, 2 pause, 3 clear, 4 game over.
     */
    tick(bits: number, dt: number): number;
    width(): number;
}

export function start(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_webgame_free: (a: number, b: number) => void;
    readonly start: () => void;
    readonly webgame_clip_samples: (a: number, b: number) => [number, number];
    readonly webgame_height: (a: number) => number;
    readonly webgame_new: () => number;
    readonly webgame_pixel_len: (a: number) => number;
    readonly webgame_pixels: (a: number) => number;
    readonly webgame_sample_rate: (a: number) => number;
    readonly webgame_take_sfx: (a: number) => [number, number];
    readonly webgame_tick: (a: number, b: number, c: number) => number;
    readonly webgame_width: (a: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
