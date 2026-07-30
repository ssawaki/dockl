/**
 * Roving-tabindex keyboard navigation for a group of custom (non-native) interactive
 * elements — list rows, context menu items, tabs, selectable cards, etc.
 *
 * Attach to the container: `<div use:rovingFocus={{ selector: '[data-roving-item]' }}>`.
 * Each item needs `data-roving-item` (or whatever selector is passed) so the action can
 * find it; the action manages `tabindex` itself (Tab enters/exits the group in one
 * stop, arrow keys move the "current" item within it — the standard roving-tabindex
 * pattern used by native listboxes/menus/tablists).
 *
 * Enter/Space on the focused item calls `.click()` on it, so existing `onclick`
 * handlers on rows/items/tabs "just work" for keyboard activation without every
 * component needing its own Enter/Space handler.
 */
export interface RovingFocusOptions {
  selector: string;
  orientation?: "vertical" | "horizontal";
}

export function rovingFocus(node: HTMLElement, options: RovingFocusOptions) {
  let opts = options;

  function items(): HTMLElement[] {
    return Array.from(node.querySelectorAll<HTMLElement>(opts.selector));
  }

  function setup() {
    const els = items();
    if (els.length === 0) return;
    const current = els.find((el) => el.tabIndex === 0);
    els.forEach((el, i) => {
      el.tabIndex = current ? (el === current ? 0 : -1) : i === 0 ? 0 : -1;
    });
  }

  function focusItem(el: HTMLElement) {
    items().forEach((i) => (i.tabIndex = i === el ? 0 : -1));
    el.focus();
  }

  function onKeydown(e: KeyboardEvent) {
    const els = items();
    if (els.length === 0) return;
    const currentIndex = els.indexOf(document.activeElement as HTMLElement);
    const vertical = opts.orientation !== "horizontal";
    const nextKey = vertical ? "ArrowDown" : "ArrowRight";
    const prevKey = vertical ? "ArrowUp" : "ArrowLeft";

    if (e.key === nextKey) {
      e.preventDefault();
      focusItem(els[(currentIndex + 1 + els.length) % els.length] ?? els[0]);
    } else if (e.key === prevKey) {
      e.preventDefault();
      focusItem(els[(currentIndex - 1 + els.length) % els.length] ?? els[els.length - 1]);
    } else if (e.key === "Home") {
      e.preventDefault();
      focusItem(els[0]);
    } else if (e.key === "End") {
      e.preventDefault();
      focusItem(els[els.length - 1]);
    } else if ((e.key === "Enter" || e.key === " ") && currentIndex >= 0) {
      e.preventDefault();
      els[currentIndex].click();
    }
  }

  setup();
  node.addEventListener("keydown", onKeydown);

  return {
    update(newOptions: RovingFocusOptions) {
      opts = newOptions;
      setup();
    },
    destroy() {
      node.removeEventListener("keydown", onKeydown);
    },
  };
}

/** Focuses the first roving item in a container, e.g. right after a menu opens. */
export function focusFirstRovingItem(node: HTMLElement, selector: string) {
  const first = node.querySelector<HTMLElement>(selector);
  first?.focus();
}
