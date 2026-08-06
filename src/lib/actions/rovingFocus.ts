/**
 * Roving-tabindex keyboard navigation for a group of custom (non-native) interactive
 * elements — list rows, context menu items, tabs, selectable cards, etc.
 *
 * Attach to the container: `<div use:rovingFocus={{ selector: '[data-roving-item]' }}>`.
 * Each item needs `data-roving-item` (or whatever selector is passed) so the action can
 * find it; the action manages `tabindex` itself (Tab enters/exits the group in one
 * stop, arrow keys move the "current" item within it — the standard roving-tabindex
 * pattern used by native listboxes/menus/tablists). Arrow keys stop at the first and
 * last item instead of wrapping around; Home/End jump to either end.
 *
 * Enter/Space on the focused item calls `.click()` on it, so existing `onclick`
 * handlers on rows/items/tabs "just work" for keyboard activation without every
 * component needing its own Enter/Space handler.
 */
export interface RovingFocusOptions {
  selector: string;
  orientation?: "vertical" | "horizontal";
  /**
   * Where the group's single tab stop sits while nothing is selected.
   *
   * Off (the default), it's the first item, which suits groups that always have a
   * current one — a tablist, the nav rail, a menu. On, it's the container itself, and an
   * arrow key moves focus into the list; that fits a master list, where landing on the
   * first row would look like a selection the user never made.
   */
  fallbackToContainer?: boolean;
}

export function rovingFocus(node: HTMLElement, options: RovingFocusOptions) {
  let opts = options;

  function items(): HTMLElement[] {
    return Array.from(node.querySelectorAll<HTMLElement>(opts.selector));
  }

  /**
   * The item the group marks as current, if any — however it spells that. Lists and tabs
   * use `aria-selected`, radio-style cards `aria-checked`, and links `aria-current`
   * (`aria-selected` isn't valid on a link, so the nav rail has no other way to say it).
   */
  function selected(els: HTMLElement[]): HTMLElement | undefined {
    return els.find((el) => {
      const current = el.getAttribute("aria-current");
      return (
        el.getAttribute("aria-selected") === "true" ||
        el.getAttribute("aria-checked") === "true" ||
        (current !== null && current !== "false")
      );
    });
  }

  function setup() {
    const els = items();
    if (els.length === 0) {
      node.tabIndex = -1;
      return;
    }

    // A selection outranks whatever was last focused: returning to a list should land on
    // the row whose details are on screen, not wherever the caret happened to stop.
    const target = selected(els) ?? els.find((el) => el.tabIndex === 0);

    if (!target && opts.fallbackToContainer) {
      node.tabIndex = 0;
      els.forEach((el) => (el.tabIndex = -1));
      return;
    }

    node.tabIndex = -1;
    els.forEach((el, i) => {
      el.tabIndex = target ? (el === target ? 0 : -1) : i === 0 ? 0 : -1;
    });
  }

  function focusItem(el: HTMLElement) {
    // The item becomes the group's tab stop, so the container has to give its own up or
    // Tab would stop twice on the way in.
    node.tabIndex = -1;
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
    // Focus is on the container itself, i.e. the list was entered with nothing selected.
    // The first arrow key enters from whichever end matches the direction it points.
    const outsideItems = currentIndex < 0;

    // Stops at both ends rather than wrapping, which is what a list is expected to do —
    // wrapping makes a long list feel like it silently jumped somewhere else, and there's
    // no way to tell the two apart without looking. Home/End are the way to reach the far
    // end deliberately. preventDefault() still fires at the ends so a held arrow key can't
    // fall through to scrolling the panel.
    if (e.key === nextKey) {
      e.preventDefault();
      const next = outsideItems ? 0 : currentIndex + 1;
      if (next < els.length) focusItem(els[next]);
    } else if (e.key === prevKey) {
      e.preventDefault();
      const prev = outsideItems ? els.length - 1 : currentIndex - 1;
      if (prev >= 0) focusItem(els[prev]);
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

  // Every list here fills in asynchronously — the rows don't exist yet when the action
  // mounts, so a one-shot setup() finds nothing, returns early, and leaves every row at
  // the tabindex="-1" it was authored with. The group then has no tab stop at all and Tab
  // skips straight past it, which is what made the volume/network/container lists
  // unreachable by keyboard. Watching the selection attributes too keeps the tab stop on
  // the current item as the selection moves — including when it moves by mouse, which
  // never goes through focusItem(). `tabindex` is deliberately not watched, so the writes
  // setup() itself makes can't feed back into this.
  const observer = new MutationObserver(setup);
  observer.observe(node, {
    childList: true,
    subtree: true,
    attributeFilter: ["aria-selected", "aria-checked", "aria-current"],
  });

  setup();
  node.addEventListener("keydown", onKeydown);

  return {
    update(newOptions: RovingFocusOptions) {
      opts = newOptions;
      setup();
    },
    destroy() {
      observer.disconnect();
      node.removeEventListener("keydown", onKeydown);
    },
  };
}

/** Focuses the first roving item in a container, e.g. right after a menu opens. */
export function focusFirstRovingItem(node: HTMLElement, selector: string) {
  const first = node.querySelector<HTMLElement>(selector);
  first?.focus();
}
