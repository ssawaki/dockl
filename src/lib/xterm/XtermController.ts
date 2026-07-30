import { Terminal, type ITheme } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { SearchAddon } from "@xterm/addon-search";

const lightTheme: ITheme = {
  background: "#00000000",
  foreground: "#1b1a19",
  cursor: "#1b1a19",
  selectionBackground: "#0f6cbd55",
};

const darkTheme: ITheme = {
  background: "#00000000",
  foreground: "#f3f2f1",
  cursor: "#f3f2f1",
  selectionBackground: "#0f6cbd55",
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
  private resizeObserver: ResizeObserver | null = null;
  private mediaQuery: MediaQueryList | null = null;

  constructor(options: { interactive?: boolean } = {}) {
    this.terminal = new Terminal({
      convertEol: !options.interactive,
      scrollback: 10000,
      fontFamily: "Cascadia Code, Consolas, 'Segoe UI Mono', monospace",
      fontSize: 13,
      theme: this.currentTheme(),
      disableStdin: !options.interactive,
      cursorBlink: options.interactive,
    });
    this.terminal.loadAddon(this.fitAddon);
    this.terminal.loadAddon(this.searchAddon);
  }

  private currentTheme(): ITheme {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? darkTheme : lightTheme;
  }

  mount(el: HTMLElement) {
    this.terminal.open(el);

    // `fit()` measures `el`'s current layout box, which right after `open()` in the
    // same tick can still reflect a pre-layout size (e.g. mid tab-switch), undersizing
    // the terminal and clipping the last row. Deferring one frame lets the browser
    // finish layout first; the ResizeObserver below keeps it correct after that.
    requestAnimationFrame(() => this.fit());

    this.resizeObserver = new ResizeObserver(() => this.fit());
    this.resizeObserver.observe(el);

    this.mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    this.mediaQuery.addEventListener("change", this.applyTheme);
  }

  private applyTheme = () => {
    this.terminal.options.theme = this.currentTheme();
  };

  fit() {
    try {
      this.fitAddon.fit();
    } catch {
      // Fires harmlessly if the element isn't laid out yet (e.g. a hidden tab).
    }
  }

  /** Appends one already-line-split chunk (log viewer use case). */
  writeLine(line: string) {
    this.terminal.writeln(line);
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

  dispose() {
    this.resizeObserver?.disconnect();
    this.mediaQuery?.removeEventListener("change", this.applyTheme);
    this.terminal.dispose();
  }
}
