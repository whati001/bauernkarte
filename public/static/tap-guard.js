// Stops a tap on a dropdown option from also tapping what lies beneath.
//
// The select and combobox primitives pick an option on `pointerup` and
// close the list right away. On a touchscreen the browser follows up with
// its compatibility `click` at the same spot — by then on whatever the
// list was covering: a product chip in the navbar (which then replaces
// the product just picked) or a result card (which opens that store).
//
// Cancelling the touch's `touchend` is how a page tells the browser to
// skip that click. The listener goes on the option itself: it's already
// removed from the page when `touchend` fires, so the event no longer
// bubbles up to `document`.
document.addEventListener(
  "touchstart",
  (e) => {
    const option = e.target.closest?.("[role=option]");
    if (!option) return;
    option.addEventListener("touchend", (end) => end.cancelable && end.preventDefault(), {
      once: true,
      passive: false,
    });
  },
  { capture: true, passive: true },
);
