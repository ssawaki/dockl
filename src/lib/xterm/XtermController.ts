import { Terminal, type ITheme } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { SearchAddon } from "@xterm/addon-search";
import { copyToClipboard } from "$lib/clipboard";
import { isDarkNow } from "$lib/stores/appearance";

// `theme.background`'s alpha stays 00 so the real (Mica) page shows through the terminal,
// but its *RGB* channels still have to match the page underneath, because xterm derives
// several other colors from this value while ignoring its alpha entirely — it has no way
// to see what's actually painted behind the terminal:
//   - reverse video (`ESC[7m`) draws text in `color.opaque(background)` on a `foreground`
//     background (DomRenderer's INVERTED_DEFAULT_COLOR rules),
//   - the selection overlay div is `color.blend(background, selectionBackground)`,
//   - the character under a block cursor is `blend(background, cursorAccent)`.
// With the RGB left at 0,0,0 all three resolved to black, which in light mode meant black
// text on near-black — that's what made pasted text unreadable, since zsh highlights the
// pasted region with standout (= reverse video) by default via `zle_highlight`.
//
// The light value is One Half Light's own background; the dark one is `--dockl-solid-bg`
// from src/lib/styles/theme.css pushed a step darker, because reverse video is the one
// place this color becomes *text* rather than a backdrop and #202020 doesn't read as black
// enough against the inverted bar.
const LIGHT_BACKGROUND = "#fafafa00";
const DARK_BACKGROUND = "#0c0c0c00";

// Because the selection overlay is blended against the (now correct) background RGB,
// these can be plain translucent accents — a lower alpha now genuinely means paler.
// They must not be fully opaque, though: xterm forces an opaque selectionBackground down
// to 30% alpha internally (ThemeService, xterm.js#2737), which would ignore the tuning here.
const LIGHT_SELECTION = "#0f6cbd59";
const DARK_SELECTION = "#4aa3ff59";

// xterm's built-in ANSI palette (Tango) assumes a dark background — its bright variants
// (#8ae234 green, #fce94f yellow, #eeeeec white…) are unreadable on a light page, so each
// theme brings its own 16 colors. The light set is One Half Light, verbatim; the dark set
// is Campbell (the Windows Terminal default), which is what a Windows shell's output is
// authored against.
const lightAnsi = {
  black: "#383a42",
  red: "#e45649",
  green: "#50a14f",
  yellow: "#c18401",
  blue: "#0184bc",
  magenta: "#a626a4",
  cyan: "#0997b3",
  // One Half Light keeps its two whites near the page color, so `ESC[37m`/`ESC[97m` text
  // is faint-to-invisible here — that's the upstream palette, not a transcription slip.
  white: "#bababa",
  brightBlack: "#4f525e",
  brightRed: "#e06c75",
  brightGreen: "#98c379",
  brightYellow: "#d8b36e",
  brightBlue: "#61afef",
  brightMagenta: "#c678dd",
  brightCyan: "#56b6c2",
  brightWhite: "#ffffff",
};

const darkAnsi = {
  black: "#0c0c0c",
  red: "#c50f1f",
  green: "#13a10e",
  yellow: "#c19c00",
  // Campbell's own #0037da is near-invisible on a #202020 page; lifted toward its bright
  // variant while staying clearly distinct from it.
  blue: "#3f6fe0",
  magenta: "#a83fb8",
  cyan: "#3a96dd",
  white: "#cccccc",
  brightBlack: "#767676",
  brightRed: "#e74856",
  brightGreen: "#16c60c",
  brightYellow: "#f9f1a5",
  brightBlue: "#3b78ff",
  brightMagenta: "#d670d6",
  brightCyan: "#61d6d6",
  brightWhite: "#f2f2f2",
};

const lightTheme: ITheme = {
  background: LIGHT_BACKGROUND,
  foreground: "#383a42",
  // One Half Light's own cursor is a pale lavender (#a5b4e5); this darker grey reads
  // better against the light page.
  cursor: "#4f525d",
  // The glyph *under* a block cursor — the page color, so it stays legible inside the
  // dark cursor block (xterm's default here is black, which would not).
  cursorAccent: LIGHT_BACKGROUND,
  selectionBackground: LIGHT_SELECTION,
  ...lightAnsi,
};

const darkTheme: ITheme = {
  background: DARK_BACKGROUND,
  foreground: "#f3f2f1",
  cursor: "#f3f2f1",
  cursorAccent: DARK_BACKGROUND,
  selectionBackground: DARK_SELECTION,
  ...darkAnsi,
};

/**
 * Thin wrapper around an xterm.js `Terminal`, shared by the read-only log viewer and
 * the interactive attach/exec view (both want the same fit/theme/dispose plumbing;
 * `interactive: true` just enables stdin for the latter).
 */
export class XtermController {
  readonly terminal: Terminal;
  private readonly fitAddon = new FitAddon();
  readonly searchAddon = new SearchAddon();
  private readonly interactive: boolean;
  private resizeObserver: ResizeObserver | null = null;
  private hostEl: HTMLElement | null = null;

  constructor(options: { interactive?: boolean } = {}) {
    this.interactive = options.interactive ?? false;
    this.terminal = new Terminal({
      // SearchAddon's decorations (match highlighting) use xterm's decoration/marker
      // API, which is still gated behind this flag — without it, `findNext`/`findPrevious`
      // throw as soon as `searchOptions.decorations` is passed, so search silently never
      // ran at all (this is what made "matches nothing" look identical across several
      // unrelated fix attempts: the call was erroring out before it could match anything).
      allowProposedApi: true,
      convertEol: !options.interactive,
      scrollback: 10000,
      fontFamily: "Cascadia Code, Consolas, 'Segoe UI Mono', monospace",
      fontSize: 13,
      theme: this.currentTheme(),
      disableStdin: !options.interactive,
      cursorBlink: options.interactive,
      // Without this, xterm ignores the alpha channel in theme.background entirely and
      // paints it opaquely, hiding the Mica page behind the terminal.
      allowTransparency: true,
      // Deliberately left at its default of 1 (off). It looks like a free readability
      // backstop for colors we don't control, but any value above 1 breaks reverse video:
      // in that path DomRendererRowFactory passes the *un*-inverted `foreground` as both
      // the background and the foreground, so the ratio is always 1, the correction always
      // fires, and it replaces the INVERTED_DEFAULT_COLOR class with a flat mid-grey —
      // ignoring `theme.background` (and so the tuning above) entirely.
      // minimumContrastRatio: <anything but 1>,
    });
    this.terminal.loadAddon(this.fitAddon);
    this.terminal.loadAddon(this.searchAddon);
    // Ctrl+C: copy the selection if there is one (like Windows Terminal/VS Code),
    // otherwise fall through to xterm's default handling (sends ^C to the pty in
    // interactive mode; a no-op when stdin is disabled, i.e. the log viewer).
    // Ctrl+V and Ctrl+Shift+V both paste, matching Windows Terminal/VS Code — this
    // does shadow plain Ctrl+V's rarely-used readline "quoted insert" binding, but
    // paste is what most people expect from it.
    // Ctrl+Backspace: delete the previous word. There's no standard terminal byte
    // sequence for this combination itself, so — matching what Windows Terminal does
    // — it's translated to ^W (0x17), the real byte a Ctrl+W keypress would send and
    // the readline binding (`unix-word-rubout`) shells already use for exactly this.
    // Leaves literal Ctrl+W itself untouched.
    this.terminal.attachCustomKeyEventHandler(this.handleCustomKeyEvent);
  }

  private handleCustomKeyEvent = (event: KeyboardEvent): boolean => {
    if (event.type !== "keydown" || !event.ctrlKey || event.altKey || event.metaKey) return true;

    const key = event.key.toLowerCase();
    if (key === "c" && !event.shiftKey && this.terminal.hasSelection()) {
      void copyToClipboard(this.terminal.getSelection());
      return false;
    }
    if (key === "v" && this.interactive) {
      // Returning `false` only tells xterm not to process this keydown itself — it
      // doesn't stop the browser's own default action for Ctrl+V on a focused text
      // input, which is to fire a native `paste` DOM event. xterm.js listens for that
      // event too (so plain OS/right-click paste works without any app code), so
      // without this, our own `pasteFromClipboard()` and xterm's native handler both
      // fire from the same keypress — pasting everything twice.
      event.preventDefault();
      void this.pasteFromClipboard();
      return false;
    }
    if (key === "backspace" && !event.shiftKey && this.interactive) {
      this.terminal.input("\x17");
      return false;
    }
    if (key === "f" && !event.shiftKey) {
      event.preventDefault();
      this.searchRequestedHandler?.();
      return false;
    }
    return true;
  };

  private searchRequestedHandler: (() => void) | null = null;

  /** Ctrl+F within the terminal calls this — wire it up to open a search UI. */
  onSearchRequested(handler: () => void) {
    this.searchRequestedHandler = handler;
  }

  private handleContextMenu = (event: MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();
    if (this.terminal.hasSelection()) {
      void copyToClipboard(this.terminal.getSelection());
      this.terminal.clearSelection();
    } else if (this.interactive) {
      void this.pasteFromClipboard();
    }
  };

  private async pasteFromClipboard() {
    try {
      const text = await navigator.clipboard.readText();
      if (text) this.terminal.paste(text);
    } catch {
      // Clipboard read can fail (denied permission, empty clipboard) — nothing to paste.
    }
  }

  private currentTheme(): ITheme {
    return isDarkNow() ? darkTheme : lightTheme;
  }

  /**
   * Resolves once the initial fit has run, so callers that need to size a PTY off
   * `cols`/`rows` (interactive attach) can wait for real dimensions instead of xterm's
   * 80x24 default — see the interactive callers' `mountTerminal` for why this matters.
   */
  mount(el: HTMLElement): Promise<void> {
    this.terminal.open(el);
    this.hostEl = el;
    el.addEventListener("contextmenu", this.handleContextMenu);

    // `fit()` measures `el`'s current layout box, which right after `open()` in the
    // same tick can still reflect a pre-layout size (e.g. mid tab-switch), undersizing
    // the terminal and clipping the last row. Deferring one frame lets the browser
    // finish layout first; the ResizeObserver below keeps it correct after that.
    const initialFit = new Promise<void>((resolve) => {
      requestAnimationFrame(() => {
        this.fit();
        resolve();
      });
    });

    this.resizeObserver = new ResizeObserver(() => this.fit());
    this.resizeObserver.observe(el);

    // Fired by src/lib/stores/appearance.ts on every resolved theme change (a manual
    // Settings switch, or the OS changing while themeMode is "system") — this isn't a
    // Svelte component, so it can't just subscribe to that store directly.
    window.addEventListener("dockl-theme-change", this.applyTheme);

    return initialFit;
  }

  private applyTheme = () => {
    this.terminal.options.theme = this.currentTheme();
  };

  // xterm.js has no client-side "don't wrap" option — wrapping is baked into how the
  // buffer lays text out at write time, driven by the terminal's column count. "No
  // wrap" is faked here the same way some terminal emulators do it: give the buffer far
  // more columns than the container can show (so real-world log lines essentially never
  // hit the wrap point) and let the container scroll horizontally instead, rather than
  // fitting columns exactly to the container's width like `wrapEnabled` mode does.
  private wrapEnabled = true;
  private static readonly NO_WRAP_COLS = 500;

  /** Only meaningful for read-only usage (log viewing) — see this field's own comment. */
  setWrapEnabled(enabled: boolean) {
    if (this.wrapEnabled === enabled) return;
    this.wrapEnabled = enabled;
    this.fit();
  }

  fit() {
    // Nothing to fit while the host isn't rendered — and, more importantly, measuring it
    // anyway is destructive. FitAddon sizes the terminal from `getComputedStyle` on the
    // host, but inside a `display: none` subtree percentages are never resolved, so this
    // host's `height: 100%` comes back as the *string* `"100%"` — which FitAddon's
    // `parseInt` happily reads as 100 pixels. That silently reflowed the WSL shell's
    // buffer down to roughly 10x5 every time a ResizeObserver notification arrived while
    // the dialog was hidden, wrecking the scrollback the hidden-not-unmounted design
    // exists to preserve. A zero-box check is what's needed here; a NaN check isn't
    // enough, because these dimensions are wrong without ever being NaN.
    if (!this.hostEl || this.hostEl.getClientRects().length === 0) return;
    try {
      if (this.wrapEnabled) {
        this.fitAddon.fit();
        return;
      }
      const proposed = this.fitAddon.proposeDimensions();
      if (!proposed) return;
      const cols = Math.max(proposed.cols, XtermController.NO_WRAP_COLS);
      const rows = proposed.rows;
      // `FitAddon.fit()` itself skips calling `terminal.resize()` when cols/rows haven't
      // actually changed (it'd be a same-value no-op resize otherwise, and `resize()`
      // recreates the renderer/viewport) — this no-wrap branch calls `resize()` directly
      // instead of going through `FitAddon.fit()`, so it needs that same guard.
      if (cols === this.terminal.cols && rows === this.terminal.rows) return;
      this.terminal.resize(cols, rows);
    } catch {
      // Fires harmlessly if the element isn't laid out yet (e.g. a hidden tab).
    }
  }

  /** Appends one already-line-split chunk (log viewer use case). */
  writeLine(line: string) {
    this.terminal.writeln(line);
  }

  /** Appends a batch of already-line-split chunks in one `write` call rather than one
   *  per line (the log viewer receives lines batched from the Rust side already — see
   *  `docker_bridge/logs.rs`'s `spawn_line_forwarder` — so this keeps that batching
   *  benefit on this side of the IPC boundary too). */
  writeLines(lines: string[]) {
    this.terminal.write(lines.join("\r\n") + "\r\n");
  }

  /** Writes a raw chunk as-is, preserving embedded cursor/control sequences (PTY use case). */
  write(data: string) {
    this.terminal.write(data);
  }

  /** Fires on user keystrokes when the terminal is interactive. */
  onData(callback: (data: string) => void) {
    return this.terminal.onData(callback);
  }

  /** Fires whenever `fit()` changes the terminal's cols/rows (e.g. on window resize). */
  onResize(callback: (size: { cols: number; rows: number }) => void) {
    return this.terminal.onResize(callback);
  }

  get cols(): number {
    return this.terminal.cols;
  }

  get rows(): number {
    return this.terminal.rows;
  }

  clear() {
    this.terminal.clear();
  }

  focus() {
    this.terminal.focus();
  }

  dispose() {
    this.resizeObserver?.disconnect();
    window.removeEventListener("dockl-theme-change", this.applyTheme);
    this.hostEl?.removeEventListener("contextmenu", this.handleContextMenu);
    this.terminal.dispose();
  }
}
